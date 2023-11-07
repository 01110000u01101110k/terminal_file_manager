use futures::{future::FutureExt, select, StreamExt};

use crossterm::cursor::RestorePosition;
use crossterm::execute;
use crossterm::event::EventStream;
use crossterm::style::ResetColor;
use crossterm::style::{Color::{Green, Black, White, Magenta, Rgb}, Colors, SetColors};
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::cursor::MoveTo;
use crossterm::event::EnableMouseCapture;
use crossterm::event::DisableMouseCapture;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEventKind;
use crossterm::event::MouseButton;
use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::event::KeyEventKind;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::BeginSynchronizedUpdate;
use crossterm::terminal::EndSynchronizedUpdate;
use crossterm::terminal::SetSize;
use crossterm::style::SetStyle;
use crossterm::style::ContentStyle;
use crossterm::style::Attributes;
use crossterm::style::Attribute;

use std::{fs, env};
use std::{io::Result, io::stdout, io::Stdout};
use std::path::Path;
use std::io::{self, Write};
use std::path::PathBuf;
use std::string::ToString;
use std::io::stdin;
use std::fs::File;
use std::fs::create_dir;
use std::fs::remove_dir;
use std::fs::remove_file;
use std::thread::sleep;
use std::time::Duration;

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
                            if target_directory.context_menu.is_open_menu {
                                if target_directory.context_menu.selected.checked_sub(1) != None {
                                    target_directory.context_menu.selected -= 1;
    
                                    clear(stdout);
                                    target_directory.print_dir_content();
                                    target_directory.context_menu.print_context_menu(
                                        target_directory.context_menu.start_context_menu_column, 
                                        target_directory.context_menu.start_context_menu_row
                                    );
                                }
                            } else {
                                if target_directory.selected.checked_sub(1) != None {
                                    target_directory.change_selected_dir_or_file(target_directory.selected - 1);
                            
                                    clear(stdout);
                                    target_directory.print_dir_content();
                                }
                            }
                        } else if key_event.code == KeyCode::Down {
                            if target_directory.context_menu.is_open_menu {
                                if target_directory.context_menu.selected + 1 < target_directory.context_menu.menu.len() {
                                    target_directory.context_menu.selected += 1;
                                    
                                    clear(stdout);
                                    target_directory.print_dir_content();
                                    target_directory.context_menu.print_context_menu(
                                        target_directory.context_menu.start_context_menu_column, 
                                        target_directory.context_menu.start_context_menu_row
                                    );
                                }
                            } else {
                                if target_directory.selected + 1 < fs::read_dir(&mut target_directory.path).unwrap().count() {
                                    target_directory.change_selected_dir_or_file(target_directory.selected + 1);
                                    
                                    clear(stdout);
                                    target_directory.print_dir_content();
                                }
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
                            if target_directory.context_menu.is_open_menu {
                                match target_directory.context_menu.menu[target_directory.context_menu.selected] {
                                    ContextMenuItems::DeleteItem => {
                                        execute!(
                                            stdout,
                                            MoveTo(0, 0),
                                            EnterAlternateScreen,
                                            DisableMouseCapture,
                                            Show
                                        ).unwrap();

                                        let mut dir = fs::read_dir(&target_directory.path).unwrap();

                                        let dir_item = dir.nth(target_directory.selected).unwrap().unwrap();

                                        if dir_item.path().is_dir() {
                                            print!("ви дійсно хочете видалити дерикторію {}? Введіть 'yes' або 'no'.", &dir_item.file_name().to_string_lossy());
                                            std::io::stdout().flush().expect("Помилка при очищенні буфера");
    
                                            let mut entered = String::new();
                                            stdin().read_line(&mut entered).expect("something went wrong when writing the name of directory");

                                            let remove_dir = remove_dir(dir_item.path().to_str().unwrap());

                                            match remove_dir {
                                                Ok(_) => {
                                                    clear(stdout);
                                                    println!("дерикторія {} була успішно видалена", &dir_item.file_name().to_string_lossy().trim());
                                                },
                                                Err(error) => {
                                                    clear(stdout);
                                                    println!("error: {}", error);
                                                }
                                            }
                                        } else {
                                            print!("ви дійсно хочете видалити файл {}? Введіть 'yes' або 'no'.", &dir_item.file_name().to_string_lossy());
                                            std::io::stdout().flush().expect("Помилка при очищенні буфера");
    
                                            let mut entered = String::new();
                                            stdin().read_line(&mut entered).expect("something went wrong when writing the name of directory");
                                            
                                            let remove_file = remove_file(dir_item.path().to_str().unwrap());

                                            match remove_file {
                                                Ok(_) => {
                                                    clear(stdout);
                                                    println!("файл {} був успішно видалений", &dir_item.file_name().to_string_lossy().trim());
                                                },
                                                Err(error) => {
                                                    clear(stdout);
                                                    println!("error: {}", error);
                                                }
                                            }
                                        }

                                        target_directory.context_menu.is_open_menu = false;

                                        sleep(Duration::from_millis(2000));

                                        execute!(
                                            stdout,
                                            MoveTo(0, 0),
                                            LeaveAlternateScreen,
                                            EnableMouseCapture, 
                                            Hide
                                        ).unwrap();

                                        clear(stdout);
                                        target_directory.print_dir_content();
                                    },
                                    ContextMenuItems::CreateDir => {
                                        execute!(
                                            stdout,
                                            /*SetStyle(
                                                ContentStyle {
                                                    foreground_color: Some(Black),
                                                    background_color: Some(Green),
                                                    underline_color: None,
                                                    attributes: Attribute::Bold.into(),
                                                }
                                            ),*/
                                            MoveTo(0, 0),
                                            EnterAlternateScreen,
                                            DisableMouseCapture,
                                            Show
                                        ).unwrap();

                                        print!("enter the name of the directory you want to create: ");
                                        std::io::stdout().flush().expect("Помилка при очищенні буфера");

                                        let mut dir_name = String::new();
                                        stdin().read_line(&mut dir_name).expect("something went wrong when writing the name of directory");

                                        let mut patch_to_new_dir = target_directory.path.clone();

                                        patch_to_new_dir.push(&dir_name);

                                        let directory = create_dir(patch_to_new_dir.as_path().to_str().unwrap().trim())/*.expect(&format!("something went wrong when trying to create a new directory with the name {}", &dir_name))*/;
                                        
                                        match directory {
                                            Ok(_) => {
                                                println!("дерикторія {} була успішно створена", &dir_name.trim());
                                            },
                                            Err(error) => {
                                                println!("error: {}", error);
                                            }
                                        }

                                        target_directory.context_menu.is_open_menu = false;

                                        sleep(Duration::from_millis(2000));

                                        execute!(
                                            stdout,
                                            MoveTo(0, 0),
                                            LeaveAlternateScreen,
                                            EnableMouseCapture, 
                                            Hide
                                        ).unwrap();

                                        clear(stdout);
                                        target_directory.print_dir_content();
                                    },
                                    ContextMenuItems::CreateFile => {
                                        execute!(
                                            stdout,
                                            MoveTo(0, 0),
                                            EnterAlternateScreen,
                                            DisableMouseCapture,
                                            Show
                                        ).unwrap();

                                        print!("enter the name of the file you want to create: ");
                                        std::io::stdout().flush().expect("Помилка при очищенні буфера");

                                        let mut file_name = String::new();
                                        stdin().read_line(&mut file_name).expect("something went wrong when writing the name of file");

                                        let mut patch_to_new_file = target_directory.path.clone();

                                        patch_to_new_file.push(&file_name);

                                        let file = File::create(&patch_to_new_file.as_path().to_str().unwrap().trim())/*.expect(&format!("something went wrong when trying to create a new file with the name {}", &file_name))*/;
                                        
                                        match file {
                                            Ok(_) => {
                                                println!("файл {} успішно створений", &file_name.trim());
                                            },
                                            Err(error) => {
                                                println!("error: {}", error);
                                            }
                                        }

                                        target_directory.context_menu.is_open_menu = false;

                                        sleep(Duration::from_millis(2000));

                                        execute!(
                                            stdout,
                                            MoveTo(0, 0),
                                            LeaveAlternateScreen,
                                            EnableMouseCapture, 
                                            Hide
                                        ).unwrap();

                                        clear(stdout);
                                        target_directory.print_dir_content();
                                    },
                                    ContextMenuItems::Copy => {

                                    },
                                    ContextMenuItems::Paste => {

                                    },
                                    ContextMenuItems::CutItem => {

                                    }
                                }
                            } else {
                                target_directory.to_next_directory();
                            
                                clear(stdout);
                                target_directory.change_selected_dir_or_file(0);
                                target_directory.print_dir_content();
                            }
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
                        if target_directory.context_menu.is_open_menu {
                            if mouse_event.row < target_directory.context_menu.start_context_menu_row ||
                                mouse_event.row > (
                                    target_directory.context_menu.start_context_menu_row + 
                                    target_directory.context_menu.menu.len() as u16 + 1
                                ) ||
                                mouse_event.column < target_directory.context_menu.start_context_menu_column ||
                                mouse_event.column > (
                                    target_directory.context_menu.start_context_menu_column + 
                                    target_directory.context_menu.largest_element_len + 1
                                )
                            {
                                target_directory.context_menu.is_open_menu = false;
                            } else {
                                // todo - зробити перевірку на більше/меньше ніж кількість пуктів контексного меню

                                if (mouse_event.row.saturating_sub(target_directory.context_menu.start_context_menu_row)) > 0 &&
                                    (mouse_event.row.saturating_sub(target_directory.context_menu.start_context_menu_row.saturating_add(1))) < 
                                    target_directory.context_menu.menu.len() as u16 
                                {
                                    target_directory.context_menu.selected = (mouse_event.row.saturating_sub(target_directory.context_menu.start_context_menu_row.saturating_add(1))) as usize;
                                }
                                
                            }
                        } else {
                            if mouse_event.row as usize <= fs::read_dir(&mut target_directory.path).unwrap().count() {
                                target_directory.change_selected_dir_or_file(mouse_event.row as usize);
                            }
                        }

                        clear(stdout);
                        target_directory.print_dir_content();

                        if target_directory.context_menu.is_open_menu {
                            target_directory.context_menu.print_context_menu(mouse_event.column, mouse_event.row);
                        }
                    } else if mouse_event.kind == MouseEventKind::Up(MouseButton::Right) {
                        if !target_directory.context_menu.is_open_menu && 
                            mouse_event.row as usize <= fs::read_dir(&mut target_directory.path).unwrap().count() &&
                            mouse_event.row == target_directory.selected as u16
                        {
                            target_directory.context_menu.print_context_menu(mouse_event.column, mouse_event.row);
                        }
                        
                    } else if mouse_event.kind == MouseEventKind::ScrollUp {
                        if target_directory.context_menu.is_open_menu {
                            if target_directory.context_menu.selected.checked_sub(1) != None {
                                target_directory.context_menu.selected -= 1;

                                clear(stdout);
                                target_directory.print_dir_content();
                                target_directory.context_menu.print_context_menu(
                                    target_directory.context_menu.start_context_menu_column, 
                                    target_directory.context_menu.start_context_menu_row
                                );
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
                                target_directory.context_menu.print_context_menu(
                                    target_directory.context_menu.start_context_menu_column, 
                                    target_directory.context_menu.start_context_menu_row
                                );
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
        //SetSize(400, 400),
        /*SetStyle(
            ContentStyle {
                foreground_color: Some(White),
                background_color: Some(Rgb {
                    r: 37,
                    g: 37,
                    b: 37,
                }),
                underline_color: None,
                attributes: Attribute::Bold.into(),
            }
        ),*/
        EnableMouseCapture, 
        Hide
    ).unwrap();

    clear(&mut stdout);
    target_directory.print_dir_content();

    loop {
        let _ = keyboard_events(&mut stdout, &mut event_stream, &mut target_directory).await;
    }
}