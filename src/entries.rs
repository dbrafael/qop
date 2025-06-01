use std::fmt::Display;

use indexmap::IndexMap;
use serde::Deserialize;

use crate::ConfigFile;

#[derive(Deserialize, Clone, Debug)]
pub struct BaseEntry {
    exec: Option<String>,
    path: Option<String>,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub highlight_accent: Option<String>,
    pub highlight: Option<String>,
    pub accent: Option<String>,
    pub faded: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawEntry {
    key: String,
    exec: Option<String>,
    path: Option<String>,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    #[serde(flatten)]
    children: IndexMap<String, RawEntry>,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub key: String,
    pub name: String,
    pub breadcrumbs: Vec<(String, String)>,
    pub exec: String,
    pub path: String,
    pub args: Vec<String>,
    pub argstr: String,
    pub env: Vec<(String, String)>,
    pub envstr: String,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{self:?}"))
    }
}

#[derive(Default, Debug)]
pub struct Lengths {
    pub key: usize,
    pub name: usize,
    pub args: usize,
    pub path: usize,
    pub exec: usize,
    pub env: usize,
}

fn to_env(env: Vec<String>) -> Vec<(String, String)> {
    env.into_iter()
        .map(|env| {
            let mut split = env.split('=');
            let env = split.next().unwrap().to_string();
            let val = split
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("=");
            (env, val)
        })
        .collect()
}

pub fn get_entries(config: ConfigFile) -> (Vec<Entry>, Lengths) {
    let base = Entry {
        key: String::new(),
        name: String::new(),
        breadcrumbs: vec![],
        exec: config.global.exec.unwrap_or_default(),
        path: config.global.path.unwrap_or_default(),
        args: config.global.args.unwrap_or_default(),
        env: config.global.env.map(to_env).unwrap_or_default(),
        argstr: "".to_string(),
        envstr: "".to_string(),
    };

    let mut lengths = Lengths::default();
    let entries = build_entries(base, config.entries, &mut lengths);
    if let Some(found) = entries.iter().find(|elt| {
        entries
            .iter()
            .any(|elt2| elt.name != elt2.name && elt2.key.starts_with(&elt.key))
    }) {
        panic!("Prefix conflict {}", found.key);
    }
    // entries.sort_by(|a, b| b.key.cmp(&a.key));
    return (entries, lengths);
}

fn build_entries(
    current: Entry,
    children: IndexMap<String, RawEntry>,
    mut lengths: &mut Lengths,
) -> Vec<Entry> {
    children
        .into_iter()
        .map(|(name, entry)| {
            let mut to_add = current.clone();
            to_add.name.push_str(&format!(" > {name}"));
            lengths.name = lengths.name.max(to_add.name.len());
            to_add.key.push_str(&entry.key);
            to_add.breadcrumbs.push((name, to_add.key.clone()));
            lengths.key = lengths.key.max(to_add.key.len());
            to_add.exec = entry.exec.unwrap_or(to_add.exec.clone());
            lengths.exec = lengths.exec.max(to_add.exec.len());
            to_add.path = entry
                .path
                .map(|p| {
                    if p.starts_with('+') {
                        format!(
                            "{}/{}",
                            to_add.path.trim_end_matches('/'),
                            p.trim_start_matches(['/', '+']),
                        )
                    } else {
                        p
                    }
                })
                .unwrap_or(to_add.path.clone());
            lengths.path = lengths.path.max(to_add.path.len());
            to_add.args.append(&mut entry.args.unwrap_or_default());
            to_add
                .env
                .append(&mut entry.env.map(to_env).unwrap_or_default());

            match entry.children.len() {
                0 => {
                    if to_add.exec == String::new() {
                        panic!("No exec command for entry {}", to_add.name)
                    }
                    to_add.argstr = to_add.args.join(" ");
                    lengths.args = lengths.args.max(to_add.argstr.len());
                    to_add.envstr = to_add
                        .env
                        .iter()
                        .map(|env| env.0.clone())
                        .collect::<Vec<String>>()
                        .join(",");
                    lengths.env = lengths.env.max(to_add.envstr.len());
                    vec![to_add.clone()]
                }
                _ => build_entries(to_add.clone(), entry.children, &mut lengths),
            }
        })
        .flatten()
        .collect()
}
