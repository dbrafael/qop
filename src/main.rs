use std::{collections::HashMap, env, fmt::Display};

#[derive(Debug)]
struct Arg {
    value: Option<String>,
    depends: HashMap<String, ArgGrouping>,
}

impl Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _indent: usize) -> std::fmt::Result {
	let indent = vec!["  "; _indent - 1].join("");
	write!(f, "{}", self.value.clone().unwrap_or("None".to_string()))?;
	if self.depends.len() > 0 {
	    writeln!(f, ":(")?;
	    for (name, val) in self.depends.iter() {
		val.fmt(f, name.clone(), _indent)?;
	    }
	    writeln!(f, "{indent})")?;
	} else {
	    writeln!(f)?;
	}
	Ok(())
    }
}


#[derive(Debug)]
enum ArgGrouping {
    Single(Arg),
    Multi(Vec<Arg>),
}

impl ArgGrouping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, name: String, _indent: usize) -> std::fmt::Result {
	let indent = vec!["  "; _indent].join("");
	write!(f, "{indent}{name}: ")?;
	match self {
	    ArgGrouping::Single(arg) => arg.fmt(f, _indent + 1)?,
	    ArgGrouping::Multi(args) => {
		writeln!(f, "[")?;
		for arg in args {
		    write!(f, "{indent}  ")?;
		    arg.fmt(f, _indent + 2)?;
		}
		writeln!(f, "{indent}]")?;
	    },
	}
	Ok(())
    }
}

impl Display for ArgGrouping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	self.fmt(f, "".to_string(), 0)
    }
}

#[derive(Clone, Debug)]
enum ArgumentDefType {
    Simple,
    Depends(Vec<ArgumentDef>),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct ArgumentDef {
    name: String,
    many: bool,
    required: bool,
    ty: ArgumentDefType,
}

#[derive(Clone, Debug)]
struct ArgPre {
    ident: String,
    value: Option<String>,
}

fn valid_argument(arg: &str) -> Result<&str, ()> {
    match arg.chars().next().ok_or(())? {
        '-' => Ok(arg.trim_start_matches('-')),
        _ => Err(()),
    }
}

fn pre_parse_arguments(args: Vec<String>) -> Result<Vec<ArgPre>, ()> {
    let mut result = Vec::new();
    let mut iter = args.iter().skip(1).peekable();
    while let Some(arg) = iter.next() {
	let ident = valid_argument(arg)?.to_string();
        match iter.peek() {
            Some(next) => {
                if let Ok(_) = valid_argument(next) {
                    result.push(ArgPre {
                        ident, 
                        value: None,
                    }); // current option has no value
                } else {
                    let next = iter.next().unwrap();
                    result.push(ArgPre {
                        ident,
                        value: Some(next.clone()),
                    }); // current option has no value
                }
            }
            None => {
                result.push(ArgPre {
                    ident,
                    value: None,
                }); // current option has no value
                return Ok(result);
            }
        }
    }
    Ok(result)
}

fn group_arguments(args: Vec<ArgPre>, groups: &Vec<ArgumentDef>) -> HashMap<String, ArgGrouping> {
    let mut result: HashMap<String, ArgGrouping> = HashMap::new();
    let mut children = Vec::new();
    for arg in args {
	let is_top_level = groups.iter().find(|def| def.name == arg.ident);
	match is_top_level {
	    Some(def) => {
		let deps = match &def.ty {
		    ArgumentDefType::Depends(deps) => Some(deps),
		    ArgumentDefType::Simple => None,
		};
		match result.get_mut(&arg.ident) {
		    Some(exists) => {
			match exists {
			    ArgGrouping::Multi(group) => {
				if !def.many {
				    panic!("argument already set {arg:?}");
				}
				group.push(Arg {
				    value: arg.value.clone(),
				    depends: match deps {
					Some(deps) => {
					    let clone = children.clone();
					    children.clear();
					    group_arguments(clone, deps) 
					}
					None => {
					    if children.len() > 0 {
						panic!("argument {arg:?} has no dependencies but was called with some");
					    }
					    HashMap::new()
					}
				    }
				})
			    },
			    _ => panic!("argument already set {arg:?}"),
			}
		    },
		    None => {
			result.insert(
			    arg.ident.clone(),
			    match def.many {
				false => ArgGrouping::Single(
				    Arg {
					value: arg.value.clone(),
					depends: match deps {
					    Some(deps) => {
						let clone = children.clone();
						children.clear();
						group_arguments(clone, deps) 
					    }
					    None => {
						if children.len() > 0 {
						    panic!("argument {arg:?} has no dependencies but was called with some");
						}
						HashMap::new()
					    }
					}
				    }
				),
				true => ArgGrouping::Multi(
				    vec![Arg {
					value: arg.value.clone(),
					depends: match deps {
					    Some(deps) => {
						let clone = children.clone();
						children.clear();
						group_arguments(clone, deps) 
					    }
					    None => {
						if children.len() > 0 {
						    panic!("argument {arg:?} has no dependencies but was called with some");
						}
						HashMap::new()
					    }
					}
				    }]
				)
			    }
			);
		    }
		};
	    }
	    None => children.push(arg.clone()),
	}
    }
    result
}

fn main() -> Result<(), ()> {
    let args = env::args().collect::<Vec<String>>();
    let args = pre_parse_arguments(args)?;

    let defs = vec![
	ArgumentDef {
	    name: "exec".to_string(),
	    many: true,
	    required: true,
	    ty: ArgumentDefType::Depends(vec![
		ArgumentDef {
		    name: "option".to_string(),
		    many: true,
		    required: true,
		    ty: ArgumentDefType::Depends(vec![
			ArgumentDef {
			    name: "not".to_string(),
			    many: false,
			    required: false,
			    ty: ArgumentDefType::Simple,
			},
			ArgumentDef {
			    name: "exact".to_string(),
			    many: false,
			    required: false,
			    ty: ArgumentDefType::Simple,
			}
		    ]),
		},
		ArgumentDef {
		    name: "help".to_string(),
		    many: false,
		    required: false,
		    ty: ArgumentDefType::Simple,
		}
	    ]),
	},
	ArgumentDef {
	    name: "help".to_string(),
	    many: false,
	    required: false,
	    ty: ArgumentDefType::Simple,
	}
    ];

    let groups = group_arguments(args, &defs);
    for (name, val) in groups {
	println!("{name}: {val}");
    }

    Ok(())
}


// cargo run -- --option "123" --option "456" --exec "abc" --help --option "789" --exec "help"
