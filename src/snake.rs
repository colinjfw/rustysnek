use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub enum Direction {
  Up,
  Down,
  Left,
  Right,
}

impl Direction {
  pub fn to_string(self) -> &'static str {
    match self {
      Direction::Up => "up",
      Direction::Down => "down",
      Direction::Left => "left",
      Direction::Right => "right",
    }
  }
}

pub fn run(b: Move) -> Direction {
  Direction::Up
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Point {
  pub x: u16,
  pub y: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Board {
  pub height: u16,
  pub width: u16,
  pub food: Vec<Point>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Snake {
  pub id: Uuid,
  pub body: Vec<Point>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Move {
  pub turn: u32,
  pub board: Board,
  pub you: Snake,
}
