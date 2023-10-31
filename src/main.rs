use futures::{future::FutureExt, select, StreamExt};

use crossterm::cursor::RestorePosition;
use crossterm::execute;
use crossterm::event::EventStream;
use crossterm::style::ResetColor;
use crossterm::style::{Color::{Green, Black, White, Magenta}, Colors, SetColors};
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::cursor::MoveTo;
use crossterm::event::EnableMouseCapture;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEventKind;
use crossterm::event::MouseButton;
use crossterm::cursor::Hide;
use crossterm::event::KeyEventKind;

use std::{fs, env};
use std::{io::Result, io::stdout, io::Stdout};
use std::path::PathBuf;
use std::string::ToString;

enum ContextMenuItems {
    DeleteItem,
    CreateDir,
    CreateFile,
    Copy,
    Paste,
    CutItem
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
        }
    }
}

struct ContextMenu {
    menu: Vec<ContextMenuItems>,
    is_open_menu: bool,
    selected: usize,
    start_context_menu_row: u16,
    start_context_menu_column: u16,
    largest_element_len: u16,
}

impl ContextMenu {
    fn new() -> Self {
        let menu = vec![
            ContextMenuItems::DeleteItem,
            ContextMenuItems::CreateDir,
            ContextMenuItems::CreateFile,
            ContextMenuItems::Copy,
            ContextMenuItems::Paste,
            ContextMenuItems::CutItem
        ];

        Self {
            menu,
            is_open_menu: false,
            selected: 0,
            start_context_menu_row: 0,
            start_context_menu_column: 0,
            largest_element_len: 0,
        }
    }

    fn draw_row_empty_spase(
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

    fn find_largest_menu_item_len(&mut self) {
        let mut largest_element: u16 = 0;

        self.menu.iter().for_each(|element| {
            let len = element.to_string().chars().count() as u16;

            if len > largest_element {
                largest_element = len;
            }
        });

        self.largest_element_len = largest_element;
    }

    fn print_context_menu(&mut self, column: u16, row: u16) {
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

struct TargetDirectory {
    path: PathBuf,
    selected: usize,
    context_menu: ContextMenu,
}

impl TargetDirectory {
    fn new() -> Self {
        let path = env::current_dir().expect("Помилка при отриманні каталогу");

        Self {
            path,
            selected: 0,
            context_menu: ContextMenu::new(),
        }
    }

    fn to_previous_directory(&mut self) {
        self.path.pop();
    }

    fn to_next_directory(&mut self) {
        let mut dir = fs::read_dir(&self.path).unwrap();

        let dir_item = dir.nth(self.selected).unwrap().unwrap();

        if self.path.join(dir_item.path()).is_dir() {
            self.path.push(dir_item.path());
        }
    }

    fn change_selected_dir_or_file(&mut self, selected: usize) {
        self.selected = selected;
    }

    fn print_dir_content(&self) {
        for entry in fs::read_dir(&self.path).unwrap().enumerate() {
            if let (index, Ok(entry)) = entry {
                if index == self.selected {
                    execute!(
                        stdout(),
                        SetColors(Colors::new(Black, Green))
                    ).unwrap();

                    println!("{}", entry.file_name().to_string_lossy());

                    execute!(
                        stdout(),
                        ResetColor
                    ).unwrap();
                } else {
                    println!("{}", entry.file_name().to_string_lossy());
                }
            }
        }
    }
}

fn clear(stdout: &mut Stdout) {
    execute!(
        stdout,
        Clear(ClearType::All), // todo - виправити використання двох типів очищення одночасно (якщо використовувати один з них нормального очищення на Windows не відбувається)
        Clear(ClearType::Purge),
        RestorePosition,
    ).unwrap();

}

async fn keyboard_events(stdout: &mut Stdout, event_stream: &mut EventStream, target_directory: &mut TargetDirectory) -> Result<()> {
    match event_stream.next().await {
        Some(Ok(event)) => {
            match event {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Release {
                        if key_event.code == KeyCode::Up {
                            if target_directory.selected.checked_sub(1) != None {
                                target_directory.change_selected_dir_or_file(target_directory.selected - 1);
                        
                                clear(stdout);
                                target_directory.print_dir_content();
                            }
                        } else if key_event.code == KeyCode::Down {
                            if target_directory.selected + 1 < fs::read_dir(&mut target_directory.path).unwrap().count() {
                                target_directory.change_selected_dir_or_file(target_directory.selected + 1);
                                
                                clear(stdout);
                                target_directory.print_dir_content();
                            }
                        } else if key_event.code == KeyCode::Left {
                            target_directory.to_previous_directory();
                            
                            clear(stdout);
                            target_directory.change_selected_dir_or_file(0);
                            target_directory.print_dir_content();
                        } else if key_event.code == KeyCode::Right {
                            target_directory.to_next_directory();
                            
                            clear(stdout);
                            target_directory.change_selected_dir_or_file(0);
                            target_directory.print_dir_content();
                        } else if key_event.code == KeyCode::Enter {
                            target_directory.to_next_directory();
                            
                            clear(stdout);
                            target_directory.change_selected_dir_or_file(0);
                            target_directory.print_dir_content();
                        } else if key_event.code == KeyCode::Esc {
                            target_directory.to_previous_directory();
                            
                            clear(stdout);
                            target_directory.change_selected_dir_or_file(0);
                            target_directory.print_dir_content();
                        } else {
                            
                        }
                    }
                },
                Event::Mouse(mouse_event) => {
                    if mouse_event.kind == MouseEventKind::Up(MouseButton::Left) {
                        if mouse_event.row as usize <= fs::read_dir(&mut target_directory.path).unwrap().count() {
                            target_directory.change_selected_dir_or_file(mouse_event.row as usize);

                            clear(stdout);
                            target_directory.print_dir_content();
                        }

                        if target_directory.context_menu.is_open_menu {
                            target_directory.context_menu.is_open_menu = false;
                        }
                    } else if mouse_event.kind == MouseEventKind::Up(MouseButton::Right) {
                        if !target_directory.context_menu.is_open_menu && mouse_event.row as usize <= fs::read_dir(&mut target_directory.path).unwrap().count() {
                            target_directory.context_menu.print_context_menu(mouse_event.column, mouse_event.row);
                        }
                        
                    } else if mouse_event.kind == MouseEventKind::ScrollUp {
                        if target_directory.context_menu.is_open_menu {
                            if target_directory.context_menu.selected.checked_sub(1) != None {
                                target_directory.context_menu.selected -= 1;

                                clear(stdout);
                                target_directory.print_dir_content();
                                target_directory.context_menu.print_context_menu(mouse_event.column, mouse_event.row);
                            }
                        } else {
                            if target_directory.selected.checked_sub(1) != None {
                                target_directory.change_selected_dir_or_file(target_directory.selected - 1);

                                clear(stdout);
                                target_directory.print_dir_content();
                            }
                        }
                    } else if mouse_event.kind == MouseEventKind::ScrollDown {
                        if target_directory.context_menu.is_open_menu {
                            if target_directory.context_menu.selected + 1 < target_directory.context_menu.menu.len() {
                                target_directory.context_menu.selected += 1;
                                
                                clear(stdout);
                                target_directory.print_dir_content();
                                target_directory.context_menu.print_context_menu(mouse_event.column, mouse_event.row);
                            }
                        } else {
                            if target_directory.selected + 1 < fs::read_dir(&mut target_directory.path).unwrap().count() {
                                target_directory.change_selected_dir_or_file(target_directory.selected + 1);

                                clear(stdout);
                                target_directory.print_dir_content();
                            }
                        }
                    } else {

                        
                    }
                },
                _ => {  }
            }
        },
        Some(Err(_)) => {
            
        }
        None => {
            
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut stdout = stdout();

    let mut event_stream = EventStream::new();

    let mut target_directory = TargetDirectory::new();

    execute!(
        stdout,
        EnableMouseCapture, 
        Hide
    ).unwrap();

    clear(&mut stdout);
    target_directory.print_dir_content();

    loop {
        let _ = keyboard_events(&mut stdout, &mut event_stream, &mut target_directory).await;
    }
}