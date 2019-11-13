pub trait Character {
    pub fn draw(&self,  con: &mut dyn Console);
    pub fn take_damage(&mut self, damage: i32, game: &mut Game);
    pub fn get_fighter(&self) -> Option<&Fighter>;
}