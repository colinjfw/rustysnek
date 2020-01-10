struct Snake {
  health: i32,
  body: Vec<(i32, i32)>,
}

struct Board {
  height: i32,
  width: i32,
  me: i32,
  snakes: Vec<Snake>,
  food: Vec<(i32, i32)>,
}
