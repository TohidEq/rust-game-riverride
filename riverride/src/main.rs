use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor::MoveTo,
    event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{size, ScrollUp, SetSize},
    ExecutableCommand, QueueableCommand,
};

struct World {
    player_c: u16,
    player_l: u16,
}

fn draw(mut sc: &Stdout, world: &World) {
    sc.queue(MoveTo(world.player_c, world.player_l));
    sc.queue(Print("P"));

    sc.flush();
}

fn main() -> std::io::Result<()> {
    //

    // init screen
    let mut sc: Stdout = stdout();

    let (cols, rows) = size()?;

    let mut world: World = World {
        player_c: (cols / 2),
        player_l: (rows - 1),
    };

    // init the game

    loop {
        // read and apply keyboard

        // pysics

        // draw

        draw(&sc, &world);
    }

    Ok(())
}
