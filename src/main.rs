use crossterm::cursor::RestorePosition;
use crossterm::execute;

/*use futures_timer::Delay;
use futures::{future::FutureExt, select, StreamExt};*/

use std::{io::Result, io::stdout, io::Stdout, io::stdin};
use crossterm::style::ResetColor;
use crossterm::style::{Color::{Green, Black}, Colors, SetColors};
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;

use std::{fs, env};

fn clear(stdout: &mut Stdout) {
    execute!(
        stdout,
        Clear(ClearType::All),
        Clear(ClearType::Purge),
        RestorePosition,
    );
}

fn print_dir_content(stdout: &mut Stdout, dir: &std::path::Path, chosen_element: usize) {
    let entries = fs::read_dir(dir).expect("Помилка при читанні вмісту каталогу");
    
    for entry in entries.enumerate() {
        if let (num, Ok(entry)) = entry {
            if num == chosen_element {
                execute!(
                    stdout,
                    SetColors(Colors::new(Green, Black))
                );
            }

            println!("{}", entry.file_name().to_string_lossy());

            if num == chosen_element {
                execute!(
                    stdout,
                    ResetColor
                );
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut stdout = stdout();

    //let mut reader = EventStream::new();

    let mut path = env::current_dir().expect("Помилка при отриманні каталогу");


    //enable_raw_mode()?;

    loop {
        print_dir_content(&mut stdout, &path, 0);

        let mut buffer = String::new();

        stdin().read_line(&mut buffer)?;

        clear(&mut stdout);
    }

    Ok(())
}