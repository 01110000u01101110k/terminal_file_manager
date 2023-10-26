use futures::{future::FutureExt, select, StreamExt};

use crossterm::cursor::RestorePosition;
use crossterm::execute;
use crossterm::event::EventStream;
use crossterm::style::ResetColor;
use crossterm::style::{Color::{Green, Black}, Colors, SetColors};
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

struct TargetDirectory {
    path: PathBuf,
    selected: usize,
}

impl TargetDirectory {
    fn new() -> Self {
        let path = env::current_dir().expect("Помилка при отриманні каталогу");

        Self {
            path,
            selected: 0,
        }
    }

    fn to_previous_directory(&mut self) {
        self.path.pop();
    }

    fn to_next_directory(&mut self) {
        let mut dir = fs::read_dir(&self.path).unwrap();

        let data = dir.nth(self.selected).unwrap().unwrap();

        self.path.push(data.path());
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
                            target_directory.change_selected_dir_or_file(target_directory.selected - 1);
                        
                            clear(stdout);
                            target_directory.print_dir_content();
                        } else if key_event.code == KeyCode::Down {
                            target_directory.change_selected_dir_or_file(target_directory.selected + 1);
    
                            clear(stdout);
                            target_directory.print_dir_content();
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

                        
                    } else if mouse_event.kind == MouseEventKind::Up(MouseButton::Right) {

                        
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