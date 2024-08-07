#![no_std]
use gstd::{collections::HashMap, exec, msg, prelude::*, ActorId};
use gword_io::*;
// 全局可变变量，存储游戏状态
static mut WORDLE: Option<Wordle> = None;

// 单词库
const BANK_OF_WORDS: [&str; 3] = ["house", "human", "horse"];

// Wordle 游戏结构
#[derive(Default)]
struct Wordle {
    games: HashMap<ActorId, String>, // 存储用户与其对应单词的映射
}

// 初始化函数，在合约部署时调用
#[no_mangle]
extern "C" fn init() {
    unsafe {
        WORDLE = Some(Wordle::default());
    }
}

// 处理消息的函数
#[no_mangle]
extern "C" fn handle() {
    // 从消息中加载 Action 类型
    let action: Action = msg::load().expect("Unable to decode message");
    // 获取全局的 WORDLE 变量
    let wordle = unsafe { WORDLE.as_mut().expect("The program is not initialized") };

    // 根据不同的 Action 执行相应的逻辑
    let reply = match action {
        Action::StartGame { user } => {
            // 获取随机单词并开始游戏
            let random_id = get_random_value(BANK_OF_WORDS.len() as u8);
            let word = BANK_OF_WORDS[random_id as usize];
            wordle.games.insert(user, word.to_string());
            Event::GameStarted { user }
        }
        Action::CheckWord { user, word } => {
            // 检查单词长度是否为 5
            if word.len() != 5 {
                panic!("The length of the word must be 5");
            }
            // 获取用户对应的单词
            let key_word = wordle
                .games
                .get(&user)
                .expect("There is no game with this user");
            let (mut matched_indices, mut key_indices) =
                (Vec::with_capacity(5), Vec::with_capacity(5));
            // 比较用户输入与目标单词
            for (i, (a, b)) in key_word.chars().zip(word.chars()).enumerate() {
                if a == b {
                    matched_indices.push(i as u8);
                } else if key_word.contains(b) {
                    key_indices.push(i as u8);
                }
            }

            Event::WordChecked {
                user,
                correct_positions: matched_indices,
                contained_in_word: key_indices,
            }
        }
    };

    // 发送回复消息
    msg::reply(reply, 0).expect("Error in sending a reply");
}

// 随机数种子
static mut SEED: u8 = 0;

// 获取随机值的函数
pub fn get_random_value(range: u8) -> u8 {
    let seed = unsafe { SEED };
    unsafe { SEED = SEED.wrapping_add(1) };
    let mut random_input: [u8; 32] = exec::program_id().into();
    random_input[0] = random_input[0].wrapping_add(seed);
    let (random, _) = exec::random(random_input).expect("Error in getting random number");
    random[0] % range
}
