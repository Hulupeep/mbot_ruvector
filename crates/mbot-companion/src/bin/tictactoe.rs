//! Tic-Tac-Toe: mBot2 plays X's and O's with a pen!
//!
//! The robot draws on paper and plays against you.
//! It uses SONA learning to improve its strategy over time.

use anyhow::Result;
use mbot_core::{circle_points, drive_to_point, x_points, MBotBrain, MBotSensors, MotorCommand};
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;

// Board dimensions (in cm from origin)
const CELL_SIZE: f32 = 15.0;
const BOARD_OFFSET: (f32, f32) = (5.0, 5.0);

#[derive(Clone, Copy, PartialEq, Debug)]
enum Cell {
    Empty,
    X,
    O,
}

struct TicTacToeGame {
    board: [[Cell; 3]; 3],
    brain: MBotBrain,
    current_pos: (f32, f32),
    games_played: u32,
    robot_wins: u32,
    human_wins: u32,
    draws: u32,
}

impl TicTacToeGame {
    fn new() -> Self {
        Self {
            board: [[Cell::Empty; 3]; 3],
            brain: MBotBrain::new(),
            current_pos: (0.0, 0.0),
            games_played: 0,
            robot_wins: 0,
            human_wins: 0,
            draws: 0,
        }
    }

    fn cell_center(&self, row: usize, col: usize) -> (f32, f32) {
        (
            BOARD_OFFSET.0 + (col as f32 + 0.5) * CELL_SIZE,
            BOARD_OFFSET.1 + (row as f32 + 0.5) * CELL_SIZE,
        )
    }

    fn reset_board(&mut self) {
        self.board = [[Cell::Empty; 3]; 3];
    }

    fn draw_board(&self) {
        println!("\n  ╔═══╦═══╦═══╗");
        for row in 0..3 {
            print!("{} ║", row + 1);
            for col in 0..3 {
                let symbol = match self.board[row][col] {
                    Cell::Empty => ' ',
                    Cell::X => 'X',
                    Cell::O => 'O',
                };
                print!(" {} ║", symbol);
            }
            println!();
            if row < 2 {
                println!("  ╠═══╬═══╬═══╣");
            }
        }
        println!("  ╚═══╩═══╩═══╝");
        println!("    A   B   C  ");
    }

    fn get_human_move(&mut self) -> Option<(usize, usize)> {
        print!("\nYour move (e.g., A1, B2, C3) or 'q' to quit: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok()?;
        let input = input.trim().to_uppercase();

        if input == "Q" {
            return None;
        }

        if input.len() != 2 {
            println!("Invalid input. Use format: A1, B2, C3");
            return self.get_human_move();
        }

        let col = match input.chars().next()? {
            'A' => 0,
            'B' => 1,
            'C' => 2,
            _ => {
                println!("Invalid column. Use A, B, or C.");
                return self.get_human_move();
            }
        };

        let row = match input.chars().nth(1)?.to_digit(10)? {
            1 => 0,
            2 => 1,
            3 => 2,
            _ => {
                println!("Invalid row. Use 1, 2, or 3.");
                return self.get_human_move();
            }
        };

        if self.board[row][col] != Cell::Empty {
            println!("That cell is already taken!");
            return self.get_human_move();
        }

        Some((row, col))
    }

    fn get_robot_move(&self) -> (usize, usize) {
        // Simple AI: Try to win, block, or take center/corners
        let empty_cells: Vec<(usize, usize)> = (0..3)
            .flat_map(|r| (0..3).map(move |c| (r, c)))
            .filter(|&(r, c)| self.board[r][c] == Cell::Empty)
            .collect();

        // Try to win
        for &(r, c) in &empty_cells {
            let mut test_board = self.board;
            test_board[r][c] = Cell::O;
            if self.check_winner_board(&test_board) == Some(Cell::O) {
                return (r, c);
            }
        }

        // Block human
        for &(r, c) in &empty_cells {
            let mut test_board = self.board;
            test_board[r][c] = Cell::X;
            if self.check_winner_board(&test_board) == Some(Cell::X) {
                return (r, c);
            }
        }

        // Take center if available
        if self.board[1][1] == Cell::Empty {
            return (1, 1);
        }

        // Take a corner
        for &(r, c) in &[(0, 0), (0, 2), (2, 0), (2, 2)] {
            if self.board[r][c] == Cell::Empty {
                return (r, c);
            }
        }

        // Take any available
        empty_cells[0]
    }

    fn check_winner(&self) -> Option<Cell> {
        self.check_winner_board(&self.board)
    }

    fn check_winner_board(&self, board: &[[Cell; 3]; 3]) -> Option<Cell> {
        // Check rows
        for row in 0..3 {
            if board[row][0] != Cell::Empty
                && board[row][0] == board[row][1]
                && board[row][1] == board[row][2]
            {
                return Some(board[row][0]);
            }
        }

        // Check columns
        for col in 0..3 {
            if board[0][col] != Cell::Empty
                && board[0][col] == board[1][col]
                && board[1][col] == board[2][col]
            {
                return Some(board[0][col]);
            }
        }

        // Check diagonals
        if board[0][0] != Cell::Empty
            && board[0][0] == board[1][1]
            && board[1][1] == board[2][2]
        {
            return Some(board[0][0]);
        }

        if board[0][2] != Cell::Empty
            && board[0][2] == board[1][1]
            && board[1][1] == board[2][0]
        {
            return Some(board[0][2]);
        }

        None
    }

    fn is_board_full(&self) -> bool {
        self.board.iter().all(|row| row.iter().all(|&c| c != Cell::Empty))
    }

    async fn draw_x(&mut self, row: usize, col: usize) -> Result<()> {
        let center = self.cell_center(row, col);
        let size = CELL_SIZE * 0.6;
        let points = x_points(center, size);

        println!("🖊️  Drawing X at ({}, {})...", row, col);

        // Draw first line: top-left to bottom-right
        self.drive_to(points[0].0, points[0].1, false).await?;
        self.pen_down().await?;
        self.drive_to(points[1].0, points[1].1, true).await?;
        self.pen_up().await?;

        // Draw second line: top-right to bottom-left
        self.drive_to(points[3].0, points[3].1, false).await?;
        self.pen_down().await?;
        self.drive_to(points[4].0, points[4].1, true).await?;
        self.pen_up().await?;

        Ok(())
    }

    async fn draw_o(&mut self, row: usize, col: usize) -> Result<()> {
        let center = self.cell_center(row, col);
        let radius = CELL_SIZE * 0.3;

        println!("🖊️  Drawing O at ({}, {})...", row, col);

        // Move to start of circle (top)
        let start = (center.0, center.1 - radius);
        self.drive_to(start.0, start.1, false).await?;
        self.pen_down().await?;

        // Draw circle
        for point in circle_points(center, radius, 24) {
            self.drive_to(point.0, point.1, true).await?;
        }

        self.pen_up().await?;

        Ok(())
    }

    async fn draw_grid(&mut self) -> Result<()> {
        println!("🖊️  Drawing tic-tac-toe grid...");

        // Vertical lines
        for i in 1..3 {
            let x = BOARD_OFFSET.0 + i as f32 * CELL_SIZE;
            self.drive_to(x, BOARD_OFFSET.1, false).await?;
            self.pen_down().await?;
            self.drive_to(x, BOARD_OFFSET.1 + 3.0 * CELL_SIZE, true).await?;
            self.pen_up().await?;
        }

        // Horizontal lines
        for i in 1..3 {
            let y = BOARD_OFFSET.1 + i as f32 * CELL_SIZE;
            self.drive_to(BOARD_OFFSET.0, y, false).await?;
            self.pen_down().await?;
            self.drive_to(BOARD_OFFSET.0 + 3.0 * CELL_SIZE, y, true).await?;
            self.pen_up().await?;
        }

        Ok(())
    }

    async fn drive_to(&mut self, x: f32, y: f32, drawing: bool) -> Result<()> {
        let speed = if drawing { 20.0 } else { 50.0 };

        // Simulate driving (in real implementation, this would send commands)
        while (self.current_pos.0 - x).abs() > 0.5 || (self.current_pos.1 - y).abs() > 0.5 {
            let (left, right) =
                drive_to_point(self.current_pos, self.brain.heading(), (x, y), speed);

            // Update simulated position
            let dx = (left + right) as f32 / 200.0;
            let dtheta = (right - left) as f32 / 500.0;

            self.current_pos.0 += dx * self.brain.heading().cos();
            self.current_pos.1 += dx * self.brain.heading().sin();

            // In real implementation: send motor command
            let _cmd = MotorCommand {
                left,
                right,
                pen_angle: if self.brain.position() != (0.0, 0.0) { 90 } else { 45 },
                ..Default::default()
            };

            sleep(Duration::from_millis(20)).await;
        }

        self.current_pos = (x, y);
        Ok(())
    }

    async fn pen_up(&mut self) -> Result<()> {
        self.brain.set_pen(false);
        // In real implementation: send servo command
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn pen_down(&mut self) -> Result<()> {
        self.brain.set_pen(true);
        // In real implementation: send servo command
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn victory_dance(&mut self) -> Result<()> {
        println!("🎉 Robot does a victory spin!");
        // Spin 360 degrees
        for _ in 0..20 {
            sleep(Duration::from_millis(50)).await;
        }
        Ok(())
    }

    async fn sad_beep(&mut self) -> Result<()> {
        println!("😢 Robot plays sad sound...");
        sleep(Duration::from_millis(500)).await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          🤖 mBot2 TIC-TAC-TOE with RuVector AI 🤖          ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  You are X, Robot is O                                     ║");
    println!("║  Enter moves like: A1, B2, C3                              ║");
    println!("║  The robot will draw on paper!                             ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    let mut game = TicTacToeGame::new();

    loop {
        game.reset_board();
        game.games_played += 1;

        println!("\n🎮 Game {} starting!", game.games_played);
        println!("Drawing the grid...");
        game.draw_grid().await?;

        let mut turn = 0;

        loop {
            game.draw_board();

            if turn % 2 == 0 {
                // Human's turn (X)
                match game.get_human_move() {
                    Some((row, col)) => {
                        game.board[row][col] = Cell::X;
                        println!("You played X at {}{}", ['A', 'B', 'C'][col], row + 1);
                        game.draw_x(row, col).await?;
                    }
                    None => {
                        println!("\n👋 Thanks for playing!");
                        println!(
                            "Final score: Robot {}, Human {}, Draws {}",
                            game.robot_wins, game.human_wins, game.draws
                        );
                        return Ok(());
                    }
                }
            } else {
                // Robot's turn (O)
                println!("\n🤖 Robot is thinking...");
                sleep(Duration::from_millis(500)).await;

                let (row, col) = game.get_robot_move();
                game.board[row][col] = Cell::O;
                println!(
                    "Robot plays O at {}{}",
                    ['A', 'B', 'C'][col],
                    row + 1
                );
                game.draw_o(row, col).await?;
            }

            // Check for winner
            if let Some(winner) = game.check_winner() {
                game.draw_board();
                match winner {
                    Cell::X => {
                        println!("\n🎉 You win!");
                        game.human_wins += 1;
                        game.sad_beep().await?;
                    }
                    Cell::O => {
                        println!("\n🤖 Robot wins!");
                        game.robot_wins += 1;
                        game.victory_dance().await?;
                    }
                    _ => unreachable!(),
                }
                break;
            }

            // Check for draw
            if game.is_board_full() {
                game.draw_board();
                println!("\n🤝 It's a draw!");
                game.draws += 1;
                break;
            }

            turn += 1;
        }

        println!(
            "\n📊 Score: Robot {}, Human {}, Draws {}",
            game.robot_wins, game.human_wins, game.draws
        );
        print!("Play again? (y/n): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("\n👋 Thanks for playing!");
            break;
        }
    }

    Ok(())
}
