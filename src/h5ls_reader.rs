use std;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use regex::Regex;

pub struct H5Group {
    pub name: String,
    pub children: BTreeMap<String, H5Obj>
}

pub struct H5Dataset {
    pub name: String
}

//https://stackoverflow.com/questions/34953711/unwrap-inner-type-when-enum-variant-is-known
pub enum H5Obj{
    Group(H5Group),
    Dataset(H5Dataset)
}

impl H5Group {
    pub fn locate_group_mut(&mut self, path: &Path) -> &mut H5Group {
        let mut components = path.components();
        match components.next() {
            Some(group_name) => {
                let group_name = group_name.as_os_str().to_str().unwrap();
                self.children.get_mut(group_name).expect("Group doesn't exist.")
                    .to_group().locate_group_mut(components.as_path())
            },
            None => self
        }
    }
}

impl From<H5Group> for H5Obj {
    fn from(val: H5Group) -> Self {
        H5Obj::Group(val)
    }
}

impl H5Obj {
    fn to_group(&mut self) -> &mut H5Group {
        if let H5Obj::Group(g) = self { g }
        else { panic!("Failed to cast H5Obj into H5Group.") }
    }
}

fn subgroup(parent: &PathBuf, child: &PathBuf) -> Option<String> {
    let tmp = child.strip_prefix(parent);
    match tmp {
        Ok(rel_name) => match rel_name.components().count() {
            1usize => Some(String::from(rel_name.to_str().unwrap())),
            _ => None
        },
        Err(_) => None
    }
}

pub fn parse(fname: &PathBuf) -> std::io::Result<()> {
    let rule = Regex::new(r"^(?P<name>[^ ]+)\s+(?P<type>Group|Dataset)").unwrap();
    let file = File::open(fname)?;
    let reader = BufReader::new(file);
    let mut root = H5Group { name: String::from("/"), children: BTreeMap::new() };
    let mut spath = PathBuf::from(&root.name);
    // let test_path = PathBuf::from("/model_weights");
    for ll in reader.lines() {
        let line = ll?;
        let m = rule.captures(&line);
        match m {
            Some(captures) => {
                match &captures["type"] {
                    "Group" => {
                        let full_name = &captures["name"];
                        if full_name != "/" {
                            let full_name = PathBuf::from(full_name);
                            match subgroup(&spath, &full_name) {
                                Some(group_name) => {
                                    root.locate_group_mut(&spath).children.insert(
                                        group_name.clone(),
                                        H5Obj::from(H5Group {
                                            name: group_name.clone(),
                                            children: BTreeMap::new()
                                        }));
                                        spath.push(group_name.clone());
                                },
                                None => {
                                    spath.pop(); // trace back
                                }
                            };
                            // let temp_name = full_name.strip_prefix(base);
                            // match temp_name {
                            //     Some()
                            // }
                            // if full_name.starts_with(&spath) {
                            //     // Dealing with a child group
                            //     let parent = path.last_mut().unwrap();
                            //     parent.children.push(H5Obj::from(H5Group {
                            //         name: String::from(full_name.file_name().unwrap().to_str().expect("Cannot decode path.")),
                            //         children: Vec::new()
                            //     }));
                            //     //path.push(&mut me);
                            // }
                            
                        }
                    },
                    "Dataset" => (),
                    _ => ()
                }
            }
            None => ()
        };
        
    }
    Ok(())
}
