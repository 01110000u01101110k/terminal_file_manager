use std::{io::stdout, io::Stdout};
use std::path::PathBuf;
use std::string::ToString;

use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::style::{Color::{Black, White, Magenta}, Colors, SetColors};
use crossterm::cursor::MoveTo;

pub enum ContextMenuItems {
    DeleteItem,
    CreateDir,
    CreateFile,
    Copy,
    Paste,
    CutItem,
    Rename,
    Info,
    CopyPath
}

impl ToString for ContextMenuItems {
    fn to_string(&self) -> String {
        match self {
            ContextMenuItems::DeleteItem => {
                String::from("видалити")
            },
            ContextMenuItems::CreateDir => {
                String::from("створити папку")
            },
            ContextMenuItems::CreateFile => {
                String::from("створити файл")
            },
            ContextMenuItems::Copy => {
                String::from("скопіювати")
            },
            ContextMenuItems::Paste => {
                String::from("вставити")
            },
            ContextMenuItems::CutItem => {
                String::from("вирізати")
            },
            ContextMenuItems::Rename => {
                String::from("перейменувати")
            },
            ContextMenuItems::Info => {
                String::from("інформація про файл/папку")
            },
            ContextMenuItems::CopyPath => {
                String::from("скопіювати шлях до файлу/папки в буфер обміну")
            }
        }
    }
}

pub struct ContextMenu {
    pub menu: Vec<ContextMenuItems>,
    pub is_open_menu: bool,
    pub selected: usize,
    pub remember_way_to_selected_item: Option<PathBuf>,
    pub need_to_cut_elmenet: bool,
    pub start_context_menu_row: u16,
    pub start_context_menu_column: u16,
    pub largest_element_len: u16,
}

impl ContextMenu {
    pub fn new() -> Self {
        let menu = vec![
            ContextMenuItems::DeleteItem,
            ContextMenuItems::CreateDir,
            ContextMenuItems::CreateFile,
            ContextMenuItems::Copy,
            ContextMenuItems::Paste,
            ContextMenuItems::CutItem,
            ContextMenuItems::Rename,
            ContextMenuItems::Info,
            ContextMenuItems::CopyPath
        ];

        Self {
            menu,
            is_open_menu: false,
            selected: 0,
            remember_way_to_selected_item: None,
            need_to_cut_elmenet: false,
            start_context_menu_row: 0,
            start_context_menu_column: 0,
            largest_element_len: 0,
        }
    }

    pub fn draw_row_empty_spase(
        &self, 
        start_column_position: u16, 
        end_column_position: u16,
        row: u16
    ) {
        let mut column = start_column_position;

        while column <= end_column_position {
            execute!(
                stdout(), 
                MoveTo(
                    self.start_context_menu_column + column, 
                    self.start_context_menu_row + row
                )
            ).unwrap();

            print!(" ");

            column += 1;
        }
    }

    pub fn find_largest_menu_item_len(&mut self) {
        let mut largest_element: u16 = 0;

        self.menu.iter().for_each(|element| {
            let len = element.to_string().chars().count() as u16;

            if len > largest_element {
                largest_element = len;
            }
        });

        self.largest_element_len = largest_element;
    }

    pub fn print_context_menu(&mut self, column: u16, row: u16) {
        if !self.is_open_menu {
            self.start_context_menu_row = row;
            self.start_context_menu_column = column;

            self.find_largest_menu_item_len();
        }

        execute!(
            stdout(),
            SetColors(Colors::new(Black, White))
        ).unwrap();

        let mut row_counter = 0;

        self.draw_row_empty_spase(
            0, 
            self.largest_element_len + 1, 
            row_counter
        );

        row_counter += 1;

        for element in self.menu.iter().enumerate() {
            execute!(
                stdout(), 
                MoveTo(
                    self.start_context_menu_column, 
                    self.start_context_menu_row + row_counter
                )
            ).unwrap();

            if element.0 == self.selected {
                execute!(
                    stdout(),
                    SetColors(Colors::new(White, Magenta))
                ).unwrap();

                println!(" {}", element.1.to_string());

                self.draw_row_empty_spase(
                    (element.1.to_string().chars().count() + 1) as u16, 
                    self.largest_element_len + 1, 
                    row_counter
                );
            } else {
                execute!(
                    stdout(),
                    SetColors(Colors::new(Black, White))
                ).unwrap();

                println!(" {}", element.1.to_string());

                self.draw_row_empty_spase(
                    (element.1.to_string().chars().count() + 1) as u16, 
                    self.largest_element_len + 1, 
                    row_counter
                );
            }

            row_counter += 1;
        }

        execute!(
            stdout(),
            SetColors(Colors::new(Black, White))
        ).unwrap();

        self.draw_row_empty_spase(
            0, 
            self.largest_element_len + 1, 
            row_counter
        );

        execute!(
            stdout(),
            ResetColor
        ).unwrap();

        self.is_open_menu = true;
    }
}