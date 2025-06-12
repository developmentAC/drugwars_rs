// Note: I was going to put all these functions into separate files to make some organizational sense. I eventually decided to keep all code in one large `main.rs` file for the simplicity of tracking borrowed variables across functions. I may refactor all this code later once I have done more testing and added more features. Stay tuned!

// Import necessary crates for randomness, collections, and input/output
// 'colored' is used for colored terminal output
// 'rand' is used for random number generation (for prices, events, etc.)
// 'HashMap' is used for inventory and price tables
// 'io' and 'Write' are used for user input and flushing output
use colored::Colorize;
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::io;
use std::io::Write;

// ANSI color codes for colored terminal output
// These are used for consistent color formatting throughout the game
const COLOR_RESET: &str = "\x1b[0m";
const COLOR_YELLOW: &str = "\x1b[33m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_MAGENTA: &str = "\x1b[35m";

// // // // // // // // // // // // // // // // //
// Game constants for starting values and rules
// These can be tweaked for balancing or testing
const START_CASH: i32 = 2000; // Player's starting cash
// const START_CASH: i32 = 2000000; //value for testing
const START_SPACE: i32 = 100; // Starting trenchcoat space (inventory limit)
const START_DAYS: i32 = 30; // Number of days in the game
const LOAN_INTEREST: f32 = 0.15; // Daily loan interest rate
const LOAN_AMOUNT: i32 = 5000; // Initial loan amount
const MAX_HEALTH: i32 = 10; // Maximum health of the player
// const START_WEAPONS: i32 = 0; // Starting number of weapons.
const START_WEAPONS: i32 = 30; // Used for testing the game with weapons.
// // // // // // // // // // // // // // // // // //

mod toml_extract; // Extract and print the version information according to the toml file

// Enum representing all drug types in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Drug {
    Cocaine,
    Heroin,
    Acid,
    Weed,
    Speed,
    Ludes,
}

impl Drug {
    // For returning a vector of all drug variants
    fn all() -> Vec<Drug> {
        vec![
            Drug::Cocaine,
            Drug::Heroin,
            Drug::Acid,
            Drug::Weed,
            Drug::Speed,
            Drug::Ludes,
        ]
    }
    // For returning the display name for each drug
    fn name(&self) -> &'static str {
        match self {
            Drug::Cocaine => "Cocaine",
            Drug::Heroin => "Heroin",
            Drug::Acid => "Acid",
            Drug::Weed => "Weed",
            Drug::Speed => "Speed",
            Drug::Ludes => "Ludes",
        }
    }
}

// Enum for representing all city locations in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum City {
    Manhattan,
    Bronx,
    Brooklyn,
}

impl City {
    // For returning a vector of all city variants
    fn all() -> Vec<City> {
        vec![City::Manhattan, City::Bronx, City::Brooklyn]
    }
    // For returning the display name for each city
    fn name(&self) -> &'static str {
        match self {
            City::Manhattan => "Manhattan",
            City::Bronx => "The Bronx",
            City::Brooklyn => "Brooklyn",
        }
    }
}

// Struct for representing the player and their state
struct Player {
    cash: i32,                     // Player's current cash
    debt: i32,                     // Player's current debt
    health: i32,                   // Player's health (max 10)
    trenchcoat_space: i32,         // Max inventory space
    inventory: HashMap<Drug, i32>, // Drug inventory
    weapons: i32,                  // Number of weapons owned
    day: i32,                      // Current day
    city: City,                    // Current city
}

impl Player {
    // For creating new player with initial values (defined above)
    // Note these values can be changed to test the game
    // or to make it easier to play!
    // For example, you can set START_CASH to 2000000 to start with 2 million cash
    // or set START_SPACE to 1000 to start with 1000 trenchcoat space
    // or set MAX_HEALTH to 100 to start with 100 health
    // or set START_DAYS to 100 to have 100 days to play
    // or set LOAN_AMOUNT to 100000 to start with 100k debt
    // or set LOAN_INTEREST to 0.05 to have 5% daily interest on the loan
    // or set START_CASH to 1000000 and START_SPACE to 1000 to start with 1 million cash and 1000 space
    // or set MAX_HEALTH to 100 and START_DAYS to 100 to have 100 health and 100 days to play
    // or set START_WEAPONS to 10 so that each finger gets one!
    // But you don't need that, right?!!

    fn new() -> Self {
        let mut inventory = HashMap::new();
        for drug in Drug::all() {
            inventory.insert(drug, 0);
        }
        Player {
            cash: START_CASH,
            debt: LOAN_AMOUNT,
            health: MAX_HEALTH,
            trenchcoat_space: START_SPACE,
            inventory,
            // weapons: 0, //used for debugging ... you never know when you might need it!
            weapons: START_WEAPONS,
            day: 1,
            city: City::Manhattan,
        }
    }
    // For returning the total number of drugs carried
    fn total_drugs(&self) -> i32 {
        self.inventory.values().sum()
    }
}

// Struct for representing the overall game state
struct Game {
    player: Player,             // The player
    prices: HashMap<Drug, i32>, // Current drug prices
    rng: rand::rngs::ThreadRng, // Random number generator
}

impl Game {
    // Creates a new game with a new player and initial prices
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let prices = Game::generate_prices(&mut rng);
        Game {
            player: Player::new(),
            prices,
            rng,
        }
    }

    // For generating random prices for each drug for the current day
    fn generate_prices(rng: &mut rand::rngs::ThreadRng) -> HashMap<Drug, i32> {
        let mut prices = HashMap::new();
        for drug in Drug::all() {
            let price = match drug {
                Drug::Cocaine => rng.gen_range(1500..=30000),
                Drug::Heroin => rng.gen_range(1000..=14000),
                Drug::Acid => rng.gen_range(100..=1000),
                Drug::Weed => rng.gen_range(90..=800),
                Drug::Speed => rng.gen_range(100..=2500),
                Drug::Ludes => rng.gen_range(10..=600),
            };
            prices.insert(drug, price);
        }
        prices
    }

    // Prints the player's current status and inventory
    fn print_status(&self) {
        println!(
            "\n\t {CYAN}Day {}/{} in {}{RESET}",
            self.player.day,
            START_DAYS,
            self.player.city.name(),
            CYAN = COLOR_CYAN,
            RESET = COLOR_RESET
        );
        println!(
            "\t {YELLOW}Cash: ${}{RESET}",
            self.player.cash,
            YELLOW = COLOR_YELLOW,
            RESET = COLOR_RESET
        );
        println!(
            "\t {YELLOW}Debt: ${}{RESET}",
            self.player.debt,
            YELLOW = COLOR_YELLOW,
            RESET = COLOR_RESET
        );
        println!(
            "\t {GREEN}Health: {}/{}{RESET}",
            self.player.health,
            MAX_HEALTH,
            GREEN = COLOR_GREEN,
            RESET = COLOR_RESET
        );
        println!(
            "\t {MAGENTA}Weapons: {}{RESET}",
            self.player.weapons,
            MAGENTA = COLOR_MAGENTA,
            RESET = COLOR_RESET
        );
        println!(
            "\t Trenchcoat space: {}/{}",
            self.player.total_drugs(),
            self.player.trenchcoat_space
        );
        println!("\t Inventory:");
        print_a_line(COLOR_GREEN);
        for drug in Drug::all() {
            let qty = self.player.inventory[&drug];
            if qty > 0 {
                println!(
                    "\t {CYAN}█  {}: {}{RESET}",
                    drug.name(),
                    qty,
                    CYAN = COLOR_CYAN,
                    RESET = COLOR_RESET
                );
                print_a_line(COLOR_GREEN);
            }
        }
        println!("\n\t Current prices:");
        // Print a separator line
        print_a_line(COLOR_YELLOW);
        for drug in Drug::all() {
            println!(
                "\t {CYAN}█   {YELLOW}{}: ${}{RESET}",
                drug.name(),
                self.prices[&drug],
                CYAN = COLOR_CYAN,
                YELLOW = COLOR_YELLOW,
                RESET = COLOR_RESET
            );
        }
        print_a_line(COLOR_YELLOW);
    }

    // Advances the game by one day, updating prices and triggering random events
    fn next_day(&mut self) {
        self.player.day += 1;
        self.prices = Game::generate_prices(&mut self.rng);
        // Apply daily loan interest to the player's debt
        if self.player.debt > 0 {
            let interest = (self.player.debt as f32 * LOAN_INTEREST).ceil() as i32;
            self.player.debt += interest;
            println!(
                "\t{RED}Loan shark interest applied: +${} ({}% daily). New debt: ${}{RESET}",
                interest,
                (LOAN_INTEREST * 100.0) as i32,
                self.player.debt,
                RED = COLOR_RED,
                RESET = COLOR_RESET
            );
        }
        // Random news flashes that affect drug prices
        self.random_news_flash();
        // Random events: rival dealers or cops
        self.random_fight_event();
        // TODO: Add more random events, offers, etc.
    }

    // Random news flashes that cause sudden price changes
    fn random_news_flash(&mut self) {
        use rand::Rng;
        // 30% chance of a news flash each day
        if self.rng.gen_bool(0.3) {
            // Pick a random drug
            let drug = *Drug::all().choose(&mut self.rng).unwrap();
            // Pick a random event: 0 = price drop, 1 = price spike
            let event = self.rng.gen_range(0..=1);
            let news = match (drug, event) {
                (Drug::Cocaine, 0) => "Colombian cartel goes on vacation. Cocaine prices plummet!",
                (Drug::Cocaine, 1) => "Wall Street bonus season! Cocaine prices skyrocket!",
                (Drug::Heroin, 0) => "Yoga craze sweeps the city. Heroin prices crash!",
                (Drug::Heroin, 1) => "Hipsters discover 'vintage' heroin. Prices soar!",
                (Drug::Acid, 0) => "Bad trip at Burning Man. Acid prices tank!",
                (Drug::Acid, 1) => "Psychedelic parade! Acid prices go wild!",
                (Drug::Weed, 0) => "Police raid local dispensary. Weed prices nosedive!",
                (Drug::Weed, 1) => "The DrugCON meeting is held in NYC. Weed prices blaze up!",
                (Drug::Speed, 0) => "Caffeine is back in style. Speed prices collapse!",
                (Drug::Speed, 1) => "All-night coding hackathon! Speed prices explode!",
                (Drug::Ludes, 0) => "Wolf of Wall Street arrested. Ludes prices drop!",
                (Drug::Ludes, 1) => "Retro party! Ludes prices go through the roof!",
                _ => "Strange news in the city! Prices are acting weird!",
            };
            // Apply the price change
            let price = self.prices.get_mut(&drug).unwrap();
            if event == 0 {
                // Sudden drop: 40-70% off
                let drop = self.rng.gen_range(40..=70);
                *price = (*price as f32 * (1.0 - drop as f32 / 100.0)).max(1.0) as i32;
                println!(
                    "\t{CYAN}NEWS FLASH: {news}{RESET}",
                    CYAN = COLOR_CYAN,
                    news = news,
                    RESET = COLOR_RESET
                );
                println!(
                    "\t{YELLOW}>> {} price drops by {}%! Now: ${}{RESET}",
                    drug.name(),
                    drop,
                    price,
                    YELLOW = COLOR_YELLOW,
                    RESET = COLOR_RESET
                );
            } else {
                // Sudden rise: 50-120% up
                let rise = self.rng.gen_range(50..=120);
                *price = (*price as f32 * (1.0 + rise as f32 / 100.0)).max(1.0) as i32;
                println!(
                    "\t{CYAN}NEWS FLASH: {news}{RESET}",
                    CYAN = COLOR_CYAN,
                    news = news,
                    RESET = COLOR_RESET
                );
                println!(
                    "\t{YELLOW}>> {} price rises by {}%! Now: ${}{RESET}",
                    drug.name(),
                    rise,
                    price,
                    YELLOW = COLOR_YELLOW,
                    RESET = COLOR_RESET
                );
            }
        }
    }

    // Random fight event: rival drug dealers or cops
    fn random_fight_event(&mut self) {
        use rand::Rng;
        // 20% chance of a fight event each day
        if self.rng.gen_bool(0.2) {
            let is_cop = self.rng.gen_bool(0.5);
            if is_cop {
                println!(
                    "\t{CYAN}NEWS FLASH: Officer Hardass and his deputies are on a donut break... but spot you!{RESET}",
                    CYAN = COLOR_CYAN,
                    RESET = COLOR_RESET
                );
                self.fight_event(
                    "Cops",
                    15,
                    2,
                    vec![
                    //     "\t Officer Hardass yells: 'Freeze, scumbag!'",
                    //     "\t A deputy drops his donut and draws his gun!",
                    //     "\t The police radio blares: 'Suspect is armed and fabulous!'",
                    "\t A deputy drops his donut and draws his gun, only to trip on the sidewalk and face-plant.",
                    "\t A deputy drops his donut and draws his gun!",
                    "\t A deputy gets his badge stuck in a tree: 'This is not how I envisioned my career.'",
                    "\t A patrol car screeches to a halt: 'We have a situation... of epic proportions!'",
                    "\t A police officer tries to intimidate the suspect by using a fake mustache, but ends up looking ridiculous instead.",
                    "\t A rookie cop accidentally arrests a man who looks just like him, leading to an awkward exchange.",
                    "\t An officer claims to have 'expertly' handcuffed the suspect, only for them to easily slip out of the cuffs.",
                    "\t An officer gets stuck in the doorway of the suspect's car and has to be pulled out by two other officers.",
                    "\t Officer Bob mistakes a bag of chips for a stash of drugs and starts searching it with a magnifying glass.",
                    "\t Officer Hardass yells: 'Freeze, scumbag! But first, let me check my clipboard...'",
                    "\t Officer Hardass yells: 'Freeze, scumbag!'",
                    "\t Officer Johnson says: 'I've got you surrounded, suspect... on the other side of this building.'",
                    "\t Officer Jones shouts: 'I'm not searching you, I'm just... um... admiring your vehicle!'",
                    "\t Officer Smith barks into the mic: 'What's this? A warrant? No, no, no! I was just, uh, conducting research!'",
                    "\t The police car gets stuck in the parking lot due to the officer's ineptitude at parallel parking.",
                    "\t The police chief yells: 'Code 55: Code 55! That means we're out of donuts.'",
                    "\t The police radio blares: 'Suspect is armed and fabulous! Can we also order a box of donuts?'",
                    "\t The police radio blares: 'Suspect is armed and fabulous!'",
                    "\t The police radio crackles: 'All units, we have a report of suspicious activity... like someone eating an entire pizza by themselves.'",
                    "\t The police sirens are so loud that they shatter the suspect's sunglasses.",

                    ],
                );
            } else {
                println!(
                    "\t {CYAN}NEWS FLASH: Rival drug dealers challenge you to a turf war!{RESET}",
                    CYAN = COLOR_CYAN,
                    RESET = COLOR_RESET
                );
                self.fight_event(
                    "Rival Dealers",
                    10,
                    1,
                    vec![
                        "\t A rival yells: 'This is our block now!'",
                        "\t Someone throws a bag of oregano at you!",
                        "\t A dealer shouts: 'You call that product?'",
                        "\t Looks like someone's supply ran out... of dignity!",
                        "\t I see you're still peddling the same old trash, dude.",
                        "\t You must have misspelled ' failure' on your storefront sign!",
                        "\t I heard your product is so bad, it needs its own hazmat suit!",
                        "\t Looks like you left the competition to me... and my amazing deals!",
                        "\t Your operation looks like a 3rd-grader's art project gone wrong",
                        "\t Is that a 'Closed' sign or just a prayer?",
                        "\t I'm starting a betting pool on how long it takes for you to get shut down.",
                        "\t You know what they say: 'you can't buy happiness, but I heard they're selling it cheap at your store'",
                        "\t It looks like someone's trying out for the role of ' failed entrepreneur'... nice try!",
                        "\t Your reputation is so shot, I think it's still in rehab",
                        "\t Looks like you took the phrase 'on the rocks' too literally",
                        "\t I heard your product is so old, it's been known to be used as bookends",
                        "\t You must have hired a team of experts... at losing",
                        "\t This block? I think it's still on rent. You're just squatting",


                    ],
                );
            }
        }
    }

    // Fight event logic
    fn fight_event(
        &mut self,
        enemy: &str,
        mut enemy_health: i32,
        enemy_count: i32,
        funny_lines: Vec<&str>,
    ) {
        use rand::Rng;
        if self.player.weapons == 0 {
            println!(
                "\t {RED}You have no weapons! You try to run...{RESET}",
                RED = COLOR_RED,
                RESET = COLOR_RESET
            );
            if self.rng.gen_bool(0.5) {
                let dmg = self.rng.gen_range(2..=5);
                self.player.health -= dmg;
                println!(
                    "\t {RED}You got hurt while escaping! Lost {} health.{RESET}",
                    dmg,
                    RED = COLOR_RED,
                    RESET = COLOR_RESET
                );
            } else {
                println!(
                    "\t {GREEN}You barely escape unharmed!{RESET}",
                    GREEN = COLOR_GREEN,
                    RESET = COLOR_RESET
                );
            }
            return;
        }
        println!(
            "\t {YELLOW}Fight begins! {enemy} ({}) appear!{RESET}",
            enemy_count,
            YELLOW = COLOR_YELLOW,
            RESET = COLOR_RESET,
            enemy = enemy
        );
        let mut round = 1;
        while enemy_health > 0 && self.player.health > 0 {
            println!(
                "\n\t {CYAN}--- Round {} ---{RESET}",
                round,
                CYAN = COLOR_CYAN,
                RESET = COLOR_RESET
            );
            // Funny line
            if self.rng.gen_bool(0.5) {
                let line = funny_lines.choose(&mut self.rng).unwrap();
                println!(
                    "\t {MAGENTA}{}{RESET}",
                    line,
                    MAGENTA = COLOR_MAGENTA,
                    RESET = COLOR_RESET
                );
            }
            println!(
                "\t {YELLOW}Your health: {}{RESET}",
                self.player.health,
                YELLOW = COLOR_YELLOW,
                RESET = COLOR_RESET
            );
            println!(
                "\t {RED}{} health: {}{RESET}",
                enemy,
                enemy_health,
                RED = COLOR_RED,
                RESET = COLOR_RESET
            );
            print!("\t Do you want to (f)ight or (r)un? ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            match input.trim() {
                "f" => {
                    // Player attacks
                    let hit = self.rng.gen_range(3..=7);
                    println!(
                        "\t {GREEN}You attack and deal {} damage!{RESET}",
                        hit,
                        GREEN = COLOR_GREEN,
                        RESET = COLOR_RESET
                    );
                    enemy_health -= hit;
                    if enemy_health <= 0 {
                        println!(
                            "\t {GREEN}You defeated the {}!{RESET}",
                            enemy,
                            GREEN = COLOR_GREEN,
                            RESET = COLOR_RESET
                        );
                        let reward = self.rng.gen_range(1000..=5000) * enemy_count;
                        self.player.cash += reward;
                        println!(
                            "\t {YELLOW}Your loot ${}!{RESET}",
                            reward,
                            YELLOW = COLOR_YELLOW,
                            RESET = COLOR_RESET
                        );
                        break;
                    }
                    // Enemy attacks
                    let dmg = self.rng.gen_range(1..=5) * enemy_count;
                    println!(
                        "\t {RED}{} attacks and deals {} damage!{RESET}",
                        enemy,
                        dmg,
                        RED = COLOR_RED,
                        RESET = COLOR_RESET
                    );
                    self.player.health -= dmg;
                    if self.player.health <= 0 {
                        println!(
                            "\t {RED}You were defeated by the {}!{RESET}",
                            enemy,
                            RED = COLOR_RED,
                            RESET = COLOR_RESET
                        );
                        break;
                    }
                }
                "r" => {
                    if self.rng.gen_bool(0.5) {
                        println!(
                            "\t {GREEN}You escaped the fight!{RESET}",
                            GREEN = COLOR_GREEN,
                            RESET = COLOR_RESET
                        );
                        break;
                    } else {
                        let dmg = self.rng.gen_range(2..=6);
                        self.player.health -= dmg;
                        println!(
                            "\t {RED}You failed to escape and took {} damage!{RESET}",
                            dmg,
                            RED = COLOR_RED,
                            RESET = COLOR_RESET
                        );
                    }
                }
                _ => println!(
                    "\t {YELLOW}You hesitate...{RESET}",
                    YELLOW = COLOR_YELLOW,
                    RESET = COLOR_RESET
                ),
            }
            round += 1;
        }
    }

    // Handles player travel to a new city and advances the day
    fn travel(&mut self) {
        println!("\t Where do you want to go?");
        for (i, city) in City::all().iter().enumerate() {
            // println!("\t  {}. {}", i + 1, city.name());
            println!(
                "\t{my_colour}  {}. {}{RESET}",
                i + 1,
                city.name(),
                my_colour = COLOR_CYAN,
                RESET = COLOR_RESET
            ); //does cyan work here?
        }
        loop {
            print!("\t Enter your choice (0 to exit menu): ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            match input.trim().parse::<usize>() {
                Ok(0) => {
                    println!("\t Exiting travel menu.");
                    break;
                }
                Ok(choice) => {
                    if let Some(city) = City::all().get(choice - 1) {
                        self.player.city = *city;
                        self.next_day();
                        break;
                    } else {
                        println!(
                            "\t {RED}Invalid city choice. Please try again.{RESET}",
                            RED = COLOR_RED,
                            RESET = COLOR_RESET
                        );
                    }
                }
                Err(_) => {
                    println!(
                        "\t {RED}Invalid input. Please enter a number.{RESET}",
                        RED = COLOR_RED,
                        RESET = COLOR_RESET
                    );
                }
            }
        }
    }

    // Main menu for buying, selling, traveling, visiting the loan shark, or shopping around
    fn buy_sell(&mut self) {
        // Prompt the player for their next action
        print!(
            "\t {CYAN}Do you want to (b)uy, (s)ell, (t)ravel, visit the (l)oan shark, or (h) shop around?{RESET} ",
            CYAN = COLOR_CYAN,
            RESET = COLOR_RESET
        );
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        // Match the player's input to the corresponding action
        match input.trim() {
            "b" => self.buy(),
            "s" => self.sell(),
            "t" => self.travel(),
            "l" => self.loan_shark(),
            "h" => self.shop_around(),
            _ => println!(
                "{RED}Invalid choice.{RESET}",
                RED = COLOR_RED,
                RESET = COLOR_RESET
            ),
        }
    }

    // Shop around for trench coat upgrades or weapons
    fn shop_around(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let coat_price = rng.gen_range(1000..=4000);
        let weapon_price = rng.gen_range(1000..=4000);
        println!(
            "\t {CYAN}Welcome to the black market!{RESET}",
            CYAN = COLOR_CYAN,
            RESET = COLOR_RESET
        );
        println!("\t You can buy:");
        print_a_line(COLOR_YELLOW);
        println!(
            "\t  1. Larger trench coat (+50 space) for {YELLOW}${}{RESET}",
            coat_price,
            YELLOW = COLOR_YELLOW,
            RESET = COLOR_RESET
        );
        println!(
            "\t  2. Weapon (+1) for {YELLOW}${}{RESET}",
            weapon_price,
            YELLOW = COLOR_YELLOW,
            RESET = COLOR_RESET
        );
        println!("\t  3. Cancel");
        print!("\t Enter your choice: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "1" => {
                if self.player.cash >= coat_price {
                    self.player.cash -= coat_price;
                    self.player.trenchcoat_space += 50;
                    println!(
                        "\t {GREEN}You bought a larger trench coat! Space is now {}.{RESET}",
                        self.player.trenchcoat_space,
                        GREEN = COLOR_GREEN,
                        RESET = COLOR_RESET
                    );
                } else {
                    println!(
                        "\t {RED}Not enough cash for a larger trench coat.{RESET}",
                        RED = COLOR_RED,
                        RESET = COLOR_RESET
                    );
                }
            }
            "2" => {
                if self.player.cash >= weapon_price {
                    self.player.cash -= weapon_price;
                    self.player.weapons += 1;
                    println!(
                        "\t {GREEN}You bought a weapon! Weapons: {}.{RESET}",
                        self.player.weapons,
                        GREEN = COLOR_GREEN,
                        RESET = COLOR_RESET
                    );
                } else {
                    println!(
                        "\t {RED}Not enough cash for a weapon.{RESET}",
                        RED = COLOR_RED,
                        RESET = COLOR_RESET
                    );
                }
            }
            _ => println!("\t No purchase made."),
        }
    }

    // Visit the loan shark to pay off debt
    fn loan_shark(&mut self) {
        // Show current debt
        println!(
            "\t {MAGENTA}You owe the loan shark: ${}{RESET}",
            self.player.debt,
            MAGENTA = COLOR_MAGENTA,
            RESET = COLOR_RESET
        );
        print!("\t How much would you like to pay off? (Enter 0 to cancel) ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        // Parse the amount to pay
        if let Ok(amount) = input.trim().parse::<i32>() {
            if amount == 0 {
                // Player chose not to pay
                println!("\t No payment made.");
            } else if amount > self.player.cash {
                // Not enough cash to pay
                println!(
                    "\t {RED}You don't have enough cash to pay that much!{RESET}",
                    RED = COLOR_RED,
                    RESET = COLOR_RESET
                );
            } else if amount > self.player.debt {
                // Trying to pay more than owed
                println!(
                    "\t {RED}You don't owe that much!{RESET}",
                    RED = COLOR_RED,
                    RESET = COLOR_RESET
                );
            } else {
                // Deduct payment from cash and debt
                self.player.cash -= amount;
                self.player.debt -= amount;
                println!(
                    "\t {GREEN}You paid ${} to the loan shark. Remaining debt: ${}{RESET}",
                    amount,
                    self.player.debt,
                    GREEN = COLOR_GREEN,
                    RESET = COLOR_RESET
                );
            }
        } else {
            // Invalid input handling
            println!(
                "\t {RED}Invalid input.{RESET}",
                RED = COLOR_RED,
                RESET = COLOR_RESET
            );
        }
    }

    // Handles buying drugs from the market
    fn buy(&mut self) {
        // List available drugs and their prices
        println!(
            "\t {CYAN}Which drug do you want to buy?{RESET}",
            CYAN = COLOR_CYAN,
            RESET = COLOR_RESET
        );
        for (i, drug) in Drug::all().iter().enumerate() {
            println!(
                "\t  {YELLOW}{}. {} (${}){RESET}",
                i + 1,
                drug.name(),
                self.prices[drug],
                YELLOW = COLOR_YELLOW,
                RESET = COLOR_RESET
            );
        }
        print!("\t Enter your choice: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        // Parse the player's drug choice
        if let Ok(choice) = input.trim().parse::<usize>() {
            if let Some(drug) = Drug::all().get(choice - 1) {
                // Calculate max units that can be bought based on cash and space
                let max_by_cash = self.player.cash / self.prices[drug];
                let max_by_space = self.player.trenchcoat_space - self.player.total_drugs();
                let max_units = max_by_cash.min(max_by_space);
                // Show the player the max they can buy
                print!(
                    "\t How many units? (Max you can buy: {GREEN}{}{RESET}) ",
                    max_units,
                    GREEN = COLOR_GREEN,
                    RESET = COLOR_RESET
                );
                io::stdout().flush().unwrap();
                input.clear();
                io::stdin().read_line(&mut input).unwrap();
                // Parse the quantity to buy
                if let Ok(qty) = input.trim().parse::<i32>() {
                    let price = self.prices[drug] * qty;
                    let space = self.player.total_drugs() + qty;
                    if qty > max_units {
                        // Trying to buy more than allowed
                        println!(
                            "\t {RED}You can't buy that many units.{RESET}",
                            RED = COLOR_RED,
                            RESET = COLOR_RESET
                        );
                    } else if price > self.player.cash {
                        // Not enough cash
                        println!(
                            "\t {RED}Not enough cash.{RESET}",
                            RED = COLOR_RED,
                            RESET = COLOR_RESET
                        );
                    } else if space > self.player.trenchcoat_space {
                        // Not enough space
                        println!(
                            "\t {RED}Not enough space.{RESET}",
                            RED = COLOR_RED,
                            RESET = COLOR_RESET
                        );
                    } else {
                        // Complete the purchase
                        self.player.cash -= price;
                        *self.player.inventory.get_mut(drug).unwrap() += qty;
                        println!(
                            "\t {GREEN}Bought {} {}.{RESET}",
                            qty,
                            drug.name(),
                            GREEN = COLOR_GREEN,
                            RESET = COLOR_RESET
                        );
                    }
                }
            }
        }
    }

    // Handles selling drugs from the player's inventory
    fn sell(&mut self) {
        // List available drugs and their prices
        println!(
            "\t {CYAN}Which drug do you want to sell?{RESET}",
            CYAN = COLOR_CYAN,
            RESET = COLOR_RESET
        );
        for (i, drug) in Drug::all().iter().enumerate() {
            println!(
                "\t  {YELLOW}{}. {} (${}){RESET}",
                i + 1,
                drug.name(),
                self.prices[drug],
                YELLOW = COLOR_YELLOW,
                RESET = COLOR_RESET
            );
        }
        print!("\t Enter your choice: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        // Parse the player's drug choice
        if let Ok(choice) = input.trim().parse::<usize>() {
            if let Some(drug) = Drug::all().get(choice - 1) {
                let owned = self.player.inventory[drug];
                // Show the player the max they can sell
                print!(
                    "\t How many units? (Max you can sell: {GREEN}{}{RESET}) ",
                    owned,
                    GREEN = COLOR_GREEN,
                    RESET = COLOR_RESET
                );
                io::stdout().flush().unwrap();
                input.clear();
                io::stdin().read_line(&mut input).unwrap();
                // Parse the quantity to sell
                if let Ok(qty) = input.trim().parse::<i32>() {
                    if qty > owned {
                        // Trying to sell more than owned
                        println!(
                            "\t {RED}Not enough to sell.{RESET}",
                            RED = COLOR_RED,
                            RESET = COLOR_RESET
                        );
                    } else {
                        // Complete the sale
                        let price = self.prices[drug] * qty;
                        self.player.cash += price;
                        *self.player.inventory.get_mut(drug).unwrap() -= qty;
                        println!(
                            "\t {GREEN}Sold {} {}.{RESET}",
                            qty,
                            drug.name(),
                            GREEN = COLOR_GREEN,
                            RESET = COLOR_RESET
                        );
                    }
                }
            }
        }
    }

    // Checks if the game is over due to days, health, or debt
    fn is_game_over(&self) -> bool {
        self.player.day > START_DAYS
            || self.player.health <= 0
            || self.player.debt > 2 * LOAN_AMOUNT
    }

    // Prints the final score and cash at the end of the game
    fn print_final_score(&self) {
        let score = ((self.player.cash as f32) / 1_000_000.0 * 2.0).min(100.0);
        println!("\n\t Game Over! Final cash: ${}", self.player.cash);
        println!("\t Final score: {}/100", score.round() as i32);
    }
}

// Main function to start the game loop
fn main() {
    // Show the banner
    show_banner();

    // Display version information from the toml file
    toml_extract::main();

    let mut game = Game::new();

    // The below madness of a loop is used when debugging to remind of the test version of software. Genious, right?! 
    let mut counter: u32 = 1;
    while 0 != counter {
        // println!("\r Welcome to Drugwars!");
        colour_print("\t Welcome to Drugwars!", "cyan");
        counter -= 1;
    }

    while !game.is_game_over() {
        game.print_status();
        game.buy_sell();
    }
    game.print_final_score();
}

// Function to display the cheapish-looking banner
fn show_banner() {
    //logo design: "ticks", use "█" to replace "/\" chars, "_" replaced with space
    let banner = String::from(
        "
\t ██████╗    ██████╗    ██╗   ██╗    ██████╗    
\t ██╔══██╗   ██╔══██╗   ██║   ██║   ██╔════╝    
\t ██║  ██║   ██████╔╝   ██║   ██║   ██║  ███╗   
\t ██║  ██║   ██╔══██╗   ██║   ██║   ██║   ██║   
\t ██████╔╝   ██║  ██║   ╚██████╔╝   ╚██████╔╝   
\t ╚═════╝    ╚═╝  ╚═╝    ╚═════╝     ╚═════╝    
\t 
\t ██╗    ██╗       ███╗    ██████╗    ███████╗   
\t ██║    ██║      ████╗    ██╔══██╗   ██╔════╝   
\t ██║ █╗ ██║     ██╔██╗    ██████╔╝   ███████╗   
\t ██║███╗██║    ██╔╝██╗    ██╔══██╗   ╚════██║   
\t ╚███╔███╔╝   ███████╗    ██║  ██║   ███████║   
\t  ╚══╝╚══╝    ╚══════╝    ╚═╝  ╚═╝   ╚══════╝ 
\t 
",
    );

    // Print the banner in purple color
    colour_print(&banner, "cyan")
}

// Print colored text to the console
fn colour_print(text: &str, colour: &str) {
    match colour {
        "flush_green" => {
            print!("\x1b[2K\r"); // Clear the line and move to the beginning
            io::stdout().flush().unwrap();
            print!(" {}", text.bright_green().bold());
            io::stdout().flush().unwrap();
        }
        "green" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_green().bold());
        }
        "green_noLineFeed" => {
            print!("\x1b[2K\r");
            print!("{}", text.bright_green().bold());
        }
        "red" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_red().bold());
        }
        "cyan" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_cyan().bold());
        }
        "purple" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_purple().bold());
        }
        "purple_noLineFeed" => {
            print!("\x1b[2K\r");
            print!("{}", text.bright_purple().bold());
        }
        "blue" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_blue().bold());
        }
        "yellow" => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_yellow().bold());
        }
        "yellow_noLineFeed" => {
            print!("\x1b[2K\r");
            print!("{}", text.bright_yellow().bold());
        }
        _ => {
            print!("\x1b[2K\r");
            println!("{}", text.bright_yellow().bold());
        }
    }
}

// Function to print a line with a specific color (it seemed to be a good idea at the time...)
fn print_a_line(my_colour: &str) {
    println!(
        "\t{LINE_COLOUR}█████████████████████{RESET}",
        // YELLOW = COLOR_YELLOW,
        LINE_COLOUR = my_colour,
        RESET = COLOR_RESET
    );
}
