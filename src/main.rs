mod generator;
mod model;
mod parser;
mod handler;

use generator::Generate;

fn main() {
    let mut rng = rand::thread_rng();
    let input = r":trap.large:\$\\color\{#927500\}\{\\sf\\bf \{東京[工業芸科学海洋術農都立医歯理心魂情報環境数物化宗教文神聖皇修帝音薬国]{2}大学[デジタルアナログ]{4}創作同好会[traana]{3}P\}\}\$";
    let expr = parser::parse(input).unwrap();
    println!("{:?}", expr.generate(&mut rng));
}
