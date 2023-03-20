use std::{collections::HashMap, fs, time::{Instant, Duration}};

use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Item {
    cps: f64,
    cost: u64,
    id: u64,
    name: String,
    long_name: String,
}

#[derive(Debug, Default)]
struct State {
    // item type, count
    owned_items: HashMap<u64, u64>,
    code_lines: f64,
    items_index: HashMap<u64, Item>,
}

impl State {
    fn print(&self) {
        println!("Code lines: {}", self.code_lines);
        for (item_id, item_count) in self.owned_items.iter() {
            let item_type = self.items_index.get(item_id).unwrap();
            println!(
                "Owning {item_count} {} producing a total of {} code line per second",
                item_type.long_name,
                *item_count as f64 * item_type.cps
            )
        }
    }

    fn update(&mut self) {
        for (item_id, item_count) in self.owned_items.iter() {
            let item_type = self.items_index.get(item_id).unwrap();
            self.code_lines += *item_count as f64 * item_type.cps;
        }
    }

    fn action(
        &mut self,
        action: Action,
        item: Option<&str>,
        count: Option<u64>,
    ) -> Result<(), String> {
        match action {
            Action::Code => {
                self.code_lines += 1.;
            }
            Action::Buy => {
                let item = item.unwrap();
                let (item_id, item_type) = self
                    .items_index
                    .iter()
                    .find(|(_, i)| i.name == item)
                    .unwrap();
                let count = count.unwrap_or(1);
                if item_type.cost * count < self.code_lines.floor() as u64 {
                    self.code_lines -= (item_type.cost * count) as f64;
                    self.owned_items.entry(*item_id).and_modify(|e| *e+=count).or_insert(count);
                }
            }
            Action::Sell => todo!(),
            Action::Help => todo!(),
        }
        Ok(())
    }
}

enum Action {
    Code,
    Buy,
    Sell,
    Help,
}

impl TryFrom<&str> for Action {
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "code" => Ok(Action::Code),
            "sell" => Ok(Action::Sell),
            "buy" => Ok(Action::Buy),
            "help" => Ok(Action::Help),
            _ => Err("unknown action".to_string()),
        }
    }

    type Error = String;
}

fn main() -> rustyline::Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    let mut state = State::default();
    let items: Vec<Item> = serde_json::from_str(
        &fs::read_to_string("items.json").expect("Should have been able to read the file"),
    )
    .unwrap();
    state.items_index = items.into_iter().map(|i| (i.id, i)).collect();
    let mut last_tick =      Instant::now();
    loop {
        if last_tick.elapsed() >= Duration::from_secs(1){
            state.update();
            last_tick = Instant::now();
            continue;
        }
        let line = rl.readline("> ")?;
        println!("{line}");
        let mut line = line.split_ascii_whitespace();
        if let Some(action) = line.next() {
            let action = Action::try_from(action).unwrap();
            let item = line.next();
            let count = line.next().map(|s| s.parse::<u64>().unwrap());
            state.action(action, item, count).unwrap();
        }
        state.print()
    }
    Ok(())
}
