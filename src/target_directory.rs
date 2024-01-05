use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::style::{Color::{Green, Red, Black}, Colors, SetColors};
use crossterm::cursor::MoveTo;
use crossterm::event::EnableMouseCapture;
use crossterm::cursor::Hide;

use std::{fs, env};
use std::{io::stdout, io::Stdout};
use std::path::PathBuf;

use std::process::Command;
use std::env::consts::OS;

use crate::context_menu::{ContextMenu};
use crate::double_click_fixation::{FixingDoubleClick};

pub struct TargetDirectory {
    pub path: PathBuf,
    pub selected: usize,
    pub context_menu: ContextMenu,
    pub fixing_double_click: FixingDoubleClick
}

impl TargetDirectory {
    pub fn new() -> Self {
        let path = env::current_dir().expect("Помилка при отриманні каталогу");

        Self {
            path,
            selected: 0,
            context_menu: ContextMenu::new(),
            fixing_double_click: FixingDoubleClick::new()
        }
    }

    pub fn to_previous_directory(&mut self) {
        self.path.pop();

        self.change_selected_dir_or_file(0);
        self.fixing_double_click.released_both_clicks();
    }

    pub fn to_next_directory(&mut self) {
        let mut dir = fs::read_dir(&self.path).unwrap();

        let dir_item = dir.nth(self.selected).unwrap().unwrap();

        if self.path.join(dir_item.path()).is_dir() {
            self.path.push(dir_item.path());

            self.change_selected_dir_or_file(0);
        } else {
            self.open_file();
        }

        self.fixing_double_click.released_both_clicks();
    }

    pub fn open_file(&self) {
        let mut dir = fs::read_dir(&self.path).unwrap();

        let dir_item = dir.nth(self.selected).unwrap().unwrap();

        match OS {
            "windows" => {
                Command::new("cmd")
                    .args(&[
                        "/C", 
                        "explorer", 
                        self.path.join(dir_item.path()).to_str().unwrap()
                    ])
                    .status()
                    .expect(&format!("could not be opened {}", &dir_item.path().to_str().unwrap()));
            },
            "linux" => {
                Command::new("xdg-open")
                    .arg(self.path.join(&dir_item.path()).to_str().unwrap())
                    .status()
                    .expect(&format!("could not be opened {}", &dir_item.path().to_str().unwrap()));
            },
            _ => {
                println!("операційна система не підтримується");
            }
        }

        execute!(
            stdout(),
            MoveTo(0, 0),
            EnableMouseCapture, 
            Hide
        ).unwrap();
    }

    pub fn change_selected_dir_or_file(&mut self, selected: usize) {
        self.selected = selected;
    }

    pub fn print_dir_content(&self) {
        let dir_content = fs::read_dir(&self.path);

        let mut starting_point = 0;
        let mut terminal_size = crossterm::terminal::size().unwrap();

        if self.selected >= (terminal_size.1 - 1) as usize {
            starting_point = self.selected;
            terminal_size.1 += self.selected as u16;
        } 

        match dir_content {
            Ok(content) => {
                for entry in content.enumerate() { // todo - зробити більш гарний "ui", щось накшталт того як зробив контекстне меню (також додати якихось бордерів чи щось таке)
                    if let (index, Ok(entry)) = entry {
                        if index >= starting_point && index < (terminal_size.1 - 1) as usize {
                            if index == self.selected {
                                execute!(
                                    stdout(),
                                    SetColors(Colors::new(Black, Green))
                                ).unwrap();
            
                                println!("{}", entry.file_name().to_str().unwrap());
            
                                execute!(
                                    stdout(),
                                    ResetColor
                                ).unwrap();
                            } else {
                                println!("{}", entry.file_name().to_str().unwrap());
                            }   
                        }
                    }
                }
            },
            Err(error) => {
                execute!(
                    stdout(),
                    SetColors(Colors::new(Black, Red))
                ).unwrap();

                eprintln!("{}", error);

                execute!(
                    stdout(),
                    ResetColor
                ).unwrap();
            }
        }
    }
}