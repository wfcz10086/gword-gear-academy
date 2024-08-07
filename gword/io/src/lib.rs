#![no_std]

use gmeta::*;
use gstd::*;

// 定义 Wordle 合约的元数据
pub struct WordleMetadata;

impl Metadata for WordleMetadata {
    type Init = (); // 无初始化参数
    type Handle = InOut<Action, Event>; // 处理的操作和对应的事件
    type Others = (); // 无其他类型
    type Reply = (); // 无回复消息
    type Signal = (); // 无信号
    type State = (); // 无状态
}

// 用户可以执行的操作
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum Action {
    StartGame { user: ActorId },               // 开始新游戏
    CheckWord { user: ActorId, word: String }, // 检查猜测的单词
}

// 合约可以发出的事件
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum Event {
    GameStarted {
        user: ActorId, // 游戏开始事件
    },
    WordChecked {
        user: ActorId,              // 单词检查事件
        correct_positions: Vec<u8>, // 正确字母的位置
        contained_in_word: Vec<u8>, // 包含在单词中但位置错误的字母
    },
}
