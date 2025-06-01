use std::io::Stdout;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem},
    Terminal,
};

use crate::entries::{Entry, Lengths};

use anyhow::{bail, Result};

#[derive(Debug, Copy, Clone)]
pub struct RGBColor {
    r: u8,
    g: u8,
    b: u8,
}

impl Into<RGBColor> for &str {
    fn into(self) -> RGBColor {
        assert!(self.len() == 7);
        let c_sharp_without_c = &self[..1];
        assert!(&c_sharp_without_c == &"#");
        let r = &self[1..3];
        let g = &self[3..5];
        let b = &self[5..];
        RGBColor {
            r: u8::from_str_radix(&r, 16).expect("Not a number"),
            g: u8::from_str_radix(&g, 16).expect("Not a number"),
            b: u8::from_str_radix(&b, 16).expect("Not a number"),
        }
    }
}

impl Into<Color> for RGBColor {
    fn into(self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }
}

pub struct UiStyle {
    pub background: RGBColor,
    pub foreground: RGBColor,
    pub highlight_accent: RGBColor,
    pub highlight: RGBColor,
    pub accent: RGBColor,
    pub faded: RGBColor,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            background: "#121212".into(),
            foreground: "#cec0af".into(),
            highlight_accent: "#bf6601".into(),
            highlight: "#191511".into(),
            accent: "#663702".into(),
            faded: "#222222".into(),
        }
    }
}

pub fn render_ui(
    prompt: &String,
    entries: &Vec<Entry>,
    term: &mut Terminal<CrosstermBackend<Stdout>>,
    lengths: &Lengths,
    style: &UiStyle,
) -> Result<()> {
    let entries: Vec<(&Entry, bool)> = entries
        .iter()
        .map(|entry| (entry, entry.key.starts_with(prompt)))
        .collect();
    term.draw(|f| {
        let t_size = f.size();

        let entries: Vec<ListItem> = entries
            .iter()
            .map(|(entry, is_relevant)| {
                let (hl_key, nohl_key) = if *is_relevant {
                    let k = entry.key.clone();
                    let (a, b) = k.split_at(prompt.len());
                    (a.to_string(), b.to_string())
                } else {
                    ("".to_string(), entry.key.clone())
                };
                let span = Spans::from(
                    vec![
                        vec![
                            Span::styled(
                                hl_key,
                                Style::default()
                                    .fg(style.highlight_accent.into())
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(nohl_key, Style::default().fg(style.foreground.into())),
                            Span::raw(" ".repeat(lengths.key - entry.key.len() + 2)),
                        ],
                        entry
                            .breadcrumbs
                            .iter()
                            .enumerate()
                            .flat_map(|(idx, breadcrumb)| {
                                vec![
                                    Span::styled(
                                        format!("{}", breadcrumb.0.clone()),
                                        Style::default()
                                            .fg(
                                                if prompt.starts_with(&breadcrumb.1) && *is_relevant
                                                {
                                                    style.highlight_accent.into()
                                                } else {
                                                    style.foreground.into()
                                                },
                                            )
                                            .add_modifier(Modifier::BOLD),
                                    ),
                                    Span::styled(
                                        if idx == entry.breadcrumbs.len() - 1 {
                                            ""
                                        } else {
                                            " > "
                                        },
                                        Style::default().fg(style.faded.into()),
                                    ),
                                ]
                            })
                            .collect(),
                        vec![
                            Span::raw(" ".repeat(lengths.name - entry.name.len() + 2)),
                            Span::styled(
                                entry.exec.clone(),
                                Style::default()
                                    .fg(style.accent.into())
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" "),
                            Span::styled(
                                entry.argstr.clone(),
                                Style::default().fg(style.foreground.into()),
                            ),
                            Span::raw(" "),
                            Span::styled(
                                entry.path.clone(),
                                Style::default().fg(style.foreground.into()),
                            ),
                            Span::raw(" ".repeat(
                                (lengths.exec + lengths.args + lengths.path + 1).saturating_sub(
                                    entry.exec.len() + entry.argstr.len() + entry.path.len() + 1,
                                ),
                            )),
                            Span::raw(" "),
                            Span::styled(
                                entry.envstr.clone(),
                                Style::default().fg(style.faded.into()),
                            ),
                        ],
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<Span<'_>>>(),
                );
                ListItem::new(span).style(Style::default().bg(
                    if *is_relevant && prompt.len() > 0 {
                        style.highlight.into()
                    } else {
                        style.background.into()
                    },
                ))
            })
            .collect();
        let list = List::new(entries).block(Block::default());

        f.render_widget(list, t_size);
    })?;

    return Ok(());
}

pub fn input_ui(prompt: &mut String, entries: &Vec<Entry>) -> Result<()> {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('c') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    bail!("Stopping!")
                }
            }
            KeyCode::Backspace => {
                *prompt = prompt
                    .get(0..prompt.len().saturating_sub(1))
                    .unwrap_or_default()
                    .to_string();

                return Ok(());
            }
            KeyCode::Esc => {
                bail!("Stopping!")
            }
            _ => (),
        };
        match key.code {
            KeyCode::Char(char) => {
                let mut p = prompt.clone();
                p.push(char);
                if let Some(_found) = entries.iter().find(|e| e.key.starts_with(&p)) {
                    *prompt = p;
                };
            }
            _ => (),
        }
    }
    Ok(())
}
