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
    pub name: String,
    pub shape: Vec<usize>
}

pub enum H5Obj{
    Group(H5Group),
    Dataset(H5Dataset)
}

impl H5Group {
    pub fn locate_group_mut<P: AsRef<Path>>(&mut self, path: P) -> &mut H5Group {
        let path = path.as_ref();
        // println!("locate_group_mut {} {:#?}", &self.name, path);
        let mut components = path.components();
        if path.is_absolute() {
            if self.name == "/" {
                components.next(); // skip root
            }
            else { panic!("Absolute path cannot be traced from here."); }
        }

        let next = components.next();
        match next {
            None => self,
            Some(group_component) => {
                let group_name = group_component.as_os_str().to_str().unwrap(); 
                self.children.get_mut(group_name).expect(&format!("Group \"{}\" doesn't exist.", group_name))
                    .to_group_mut().locate_group_mut(components.as_path())
            }
        }
    }

    pub fn locate_group<P: AsRef<Path>>(&self, path: P) -> &H5Group {
        let path = path.as_ref();
        let mut components = path.components();
        if path.is_absolute() {
            if self.name == "/" {
                components.next(); // skip root
            }
            else { panic!("Absolute path cannot be traced from here."); }
        }

        let next = components.next();
        match next {
            None => self,
            Some(group_component) => {
                let group_name = group_component.as_os_str().to_str().unwrap(); 
                self.children.get(group_name).expect(&format!("Group \"{}\" doesn't exist.", group_name))
                    .to_group().locate_group(components.as_path())
            }
        }
    }

    pub fn is_group<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        match path.parent() {
            Some(parent) => {
                let name = path.file_name().unwrap().to_str().unwrap();
                self.locate_group(parent).children.get(name).unwrap().is_group()
            },
            None => true
        }
    }

    // pub fn locate<P: AsRef<Path>>(&self, path: P) -> &H5Obj {
    //     self.as_ref().locate(path)
    // }

    pub fn parse<P: AsRef<Path>>(fname: P) -> std::io::Result<H5Group> {
        let rule = Regex::new(r"^(?P<name>[^ ]+)\s+(?P<type>Group|Dataset)").unwrap();
        let file = File::open(fname)?;
        let reader = BufReader::new(file);
        let mut root = H5Group { name: String::from("/"), children: BTreeMap::new() };
        let mut spath = PathBuf::from(&root.name);
        for ll in reader.lines() {
            let line = ll?;
            let m = rule.captures(&line);
            match m {
                Some(captures) => {
                    match &captures["type"] {
                        "Group" => {
                            let full_name = &captures["name"];
                            if full_name != "/" {
                                // println!("G {}", full_name);
                                let full_name = PathBuf::from(full_name);
                                loop {
                                    match subgroup(&spath, &full_name) {
                                        Some(group_name) => {
                                            root.locate_group_mut(&spath).children.insert(
                                                group_name.clone(),
                                                H5Obj::from(H5Group {
                                                    name: group_name.clone(),
                                                    children: BTreeMap::new()
                                                }));
                                            spath.push(group_name.clone());
                                            break;
                                        },
                                        None => {
                                            spath.pop(); // trace back
                                        }
                                    }
                                }                            
                            }
                        },
                        "Dataset" => {
                            let full_name = &captures["name"];
                            let rule_dataset = Regex::new(r"^(?P<name>[^ ]+)\s+Dataset\s+\{(?P<shape>[0-9, ]*|SCALAR)\}$").unwrap();
                            let m = rule_dataset.captures(&line).expect("Malformed dataset metadata.");
                            // TODO could be scalar
                            let shape: Vec<usize> =
                                if &m["shape"] == "SCALAR" { Vec::new() }
                                else {
                                    m["shape"].split(", ")
                                    .map(|x| x.parse().expect("Error occurred when parsing dataset shape."))
                                    .collect()
                                };
                            let _format = H5Dataset::shape_to_format(&shape);
                            // println!("D {} {}", full_name, format);
                            let full_name = PathBuf::from(full_name);
                            let dataset_name = String::from(full_name.file_name().unwrap().to_str().unwrap());
                            root.locate_group_mut(&spath).children.insert(
                                dataset_name.clone(),
                                H5Obj::from(H5Dataset { name: dataset_name.clone(), shape: shape }));
                        },
                        _ => ()
                    }
                }
                None => ()
            };
            
        }
        Ok(root)
    }
}

impl H5Dataset {
    fn shape_to_format(shape: &Vec<usize>) -> String {
        String::from(
            match shape.len() {
                0 => "Param",
                1 => "Scalar",
                2 => "Vec",
                3 => "Gray",
                dims if dims >=4 =>
                    match shape.iter().last().unwrap() {
                        1 => "Gray",
                        3 => "Color",
                        4 => "RGBD",
                        _ => "Hyper"
                    }
                _ => ""
            })
    }

    pub fn format(&self) -> String {
        H5Dataset::shape_to_format(&self.shape)
    }
}

impl From<H5Group> for H5Obj {
    fn from(val: H5Group) -> Self {
        H5Obj::Group(val)
    }
}

impl From<H5Dataset> for H5Obj {
    fn from(val: H5Dataset) -> Self {
        H5Obj::Dataset(val)
    }
}

impl H5Obj {
    pub fn is_group(&self) -> bool {
        if let H5Obj::Group(g) = self { true }
        else { false }
    }

    pub fn is_dataset(&self) -> bool {
        if let H5Obj::Dataset(g) = self { true }
        else { false }
    }

    fn to_group_mut(&mut self) -> &mut H5Group {
        if let H5Obj::Group(g) = self { g }
        else { panic!("Failed to cast H5Obj into H5Group.") }
    }

    fn to_group(&self) -> &H5Group {
        if let H5Obj::Group(g) = self { g }
        else { panic!("Failed to cast H5Obj into H5Group.") }
    }

    fn name(&self) -> &str {
        match self {
            H5Obj::Group(g) => g.name.as_ref(),
            H5Obj::Dataset(d) => d.name.as_ref() 
        }
    }

    pub fn locate<P: AsRef<Path>>(&self, path: P) -> &H5Obj {
        let path = path.as_ref();
        let mut components = path.components();
        if path.is_absolute() {
            if self.name() == "/" {
                components.next(); // skip root
            }
            else { panic!("Absolute path cannot be traced from here."); }
        }

        let next = components.next();
        match next {
            None => self,
            Some(group_component) => {
                let group_name = group_component.as_os_str().to_str().unwrap(); 
                self.to_group().children.get(group_name).expect(&format!("Group \"{}\" doesn't exist.", group_name))
                    .locate(components.as_path())
            }
        }
    }
}

impl AsRef<H5Obj> for H5Group {
    fn as_ref(&self) -> &H5Obj {
        unimplemented!(); // &(H5Obj::Group(*self) as H5Obj)
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
