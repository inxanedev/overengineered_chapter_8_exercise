use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use indoc::indoc;
use serde::{Serialize, Deserialize};
use std::io;
use std::io::Write;
use std::fs;

static COMPANY_JSON_PATH: &'static str = "company.json";

#[derive(Serialize, Deserialize, Default)]
struct Company(HashMap<String, Vec<String>>);

impl Company {
    fn update_on_disk(&self) -> anyhow::Result<()> {
        Ok(fs::write(COMPANY_JSON_PATH, serde_json::to_string(&self)?)?)
    }
}

impl Deref for Company {
    type Target = HashMap<String, Vec<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Company {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

static HELP_PAGE: &'static str = indoc!{"
    Available commands:
    - 'add x to y' - adds person x to department y
    - 'remove x from y' - removes person x from department y
    - 'list department x' - lists people from department x
    - 'list company' - lists everyone by department
    - 'help' - displays help page
    - 'exit' - exits the program
"};

fn main() -> anyhow::Result<()> {
    let mut company = if !Path::new(COMPANY_JSON_PATH).exists() {
        let default_company = Company::default();
        default_company.update_on_disk()?;

        default_company
    } else {
        serde_json::from_str(&fs::read_to_string(COMPANY_JSON_PATH)?)?
    };

    println!("{}\n", HELP_PAGE);
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let tokens = input.trim().split_whitespace().collect::<Vec<&str>>();

        match tokens.as_slice() {
            ["add", x, "to", y] => {
                let x = *x;
                let y = (*y).to_lowercase();

                if !company.contains_key(&y) {
                    company.insert(y.to_lowercase(), Vec::new());
                }

                let department = company.get_mut(&y).unwrap();
                department.push((*x).to_owned());
                department.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
                company.update_on_disk()?;

                println!("Added {} to {}!", x, y);
            },
            ["remove", x, "from", y] => {
                let x = *x;
                let y = (*y).to_lowercase();

                if company.contains_key(&y) {
                    let department = company.get_mut(&y).unwrap();
                    match department.iter().position(|name| name == x) {
                        Some(position) => { department.remove(position); },
                        None => { println!("Person {} is not in {}!", x, y); }
                    };

                    if department.is_empty() {
                        company.remove(&y);
                    }

                    company.update_on_disk()?;
                } else {
                    println!("Company doesn't have department with name {}!", y);
                };
            },
            ["list", "department", x] => {
                let x = *x;
                match company.get(x) {
                    Some(department) => {
                        println!("List of people in {}", x);
                        for name in department.iter() {
                            println!("- {}", name);
                        }
                    },
                    None => println!("Department {} doesn't exist!", x)
                }
            },
            ["list", "company"] => {
                for (department_name, department) in &*company {
                    println!("List of people in {}:", department_name);
                    for name in department {
                        println!("- {}", name);
                    }
                }
            },
            ["help"] => println!("{}", HELP_PAGE),
            ["exit"] => break,
            _ => println!("Invalid command!")
        }
    }

    Ok(())
}