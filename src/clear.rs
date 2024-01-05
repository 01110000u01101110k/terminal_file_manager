use std::{io::stdout, io::Stdout};
/*
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
*/

pub fn clear(stdout: &mut Stdout) {
    /*execute!(
        stdout,
        Clear(ClearType::All), // todo - виправити використання двох типів очищення одночасно (якщо використовувати один з них нормального очищення на Windows не відбувається)
        Clear(ClearType::Purge),
        RestorePosition,
    ).unwrap();*/

    clearscreen::clear().unwrap();
}