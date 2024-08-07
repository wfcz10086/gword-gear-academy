#![no_std]

use gmeta::*;
use gstd::{collections::HashMap, prelude::*, ActorId, MessageId};

// 游戏会话元数据结构，定义合约的元数据
pub struct GameSessionMetadata;

// 元数据实现，指定合约的初始化、处理、状态等类型
impl Metadata for GameSessionMetadata {
    type Init = In<GameSessionInit>; // 初始化输入类型
    type Handle = InOut<GameSessionAction, GameSessionEvent>; // 处理动作和事件类型
    type State = Out<GameSessionState>; // 状态输出类型
    type Reply = (); // 回复类型
    type Others = (); // 其他类型
    type Signal = (); // 信号类型
}

// 游戏会话结构体
#[derive(Default, Debug, Clone)]
pub struct GameSession {
    pub wordle_program_id: ActorId,              // Wordle程序ID
    pub sessions: HashMap<ActorId, SessionInfo>, // 存储会话信息的哈希表
}

// 从游戏会话创建游戏会话状态
impl From<&GameSession> for GameSessionState {
    fn from(game_session: &GameSession) -> Self {
        Self {
            wordle_program_id: game_session.wordle_program_id,
            game_sessions: game_session
                .sessions
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect(),
        }
    }
}

// 游戏会话状态，包含Wordle程序ID和游戏会话
#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct GameSessionState {
    pub wordle_program_id: ActorId,                 // Wordle程序ID
    pub game_sessions: Vec<(ActorId, SessionInfo)>, // 游戏会话
}

// 游戏会话初始化结构体，包含Wordle程序ID
#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct GameSessionInit {
    pub wordle_program_id: ActorId, // Wordle程序ID
}

impl GameSessionInit {
    // 校验Wordle程序ID是否有效
    pub fn assert_valid(&self) {
        assert!(
            !self.wordle_program_id.is_zero(),
            "Invalid wordle_program_id"
        );
    }
}

// 从初始化创建游戏会话
impl From<GameSessionInit> for GameSession {
    fn from(game_session_init: GameSessionInit) -> Self {
        Self {
            wordle_program_id: game_session_init.wordle_program_id,
            ..Default::default()
        }
    }
}

// 游戏会话动作枚举，定义游戏中的各种动作
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum GameSessionAction {
    StartGame, // 开始游戏
    CheckWord {
        word: String, // 检查单词
    },
    CheckGameStatus {
        user: ActorId, // 检查游戏状态
        session_id: MessageId,
    },
}

// Wordle动作枚举
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum WordleAction {
    StartGame { user: ActorId },               // 开始游戏
    CheckWord { user: ActorId, word: String }, // 检查单词
}

// 游戏会话事件枚举，定义游戏中可能发生的事件
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum GameSessionEvent {
    StartSuccess, // 游戏启动成功
    CheckWordResult {
        correct_positions: Vec<u8>, // 正确位置
        contained_in_word: Vec<u8>, // 包含在单词中
    },
    GameOver(GameStatus), // 游戏结束
}

// 游戏状态枚举，定义游戏可能的结果
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum GameStatus {
    Win,  // 胜利
    Lose, // 失败
}

// Wordle事件枚举，定义Wordle游戏中可能发生的事件
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum WordleEvent {
    GameStarted {
        user: ActorId, // 游戏开始
    },
    WordChecked {
        user: ActorId, // 单词检查
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
    },
}

impl WordleEvent {
    // 获取用户ID
    pub fn get_user(&self) -> &ActorId {
        match self {
            WordleEvent::GameStarted { user } => user,
            WordleEvent::WordChecked { user, .. } => user,
        }
    }

    // 判断是否猜中
    pub fn has_guessed(&self) -> bool {
        match self {
            WordleEvent::GameStarted { .. } => unimplemented!(),
            WordleEvent::WordChecked {
                correct_positions, ..
            } => correct_positions == &vec![0, 1, 2, 3, 4],
        }
    }
}

// 从Wordle事件转换为游戏会话事件
impl From<&WordleEvent> for GameSessionEvent {
    fn from(wordle_event: &WordleEvent) -> Self {
        match wordle_event {
            WordleEvent::GameStarted { .. } => GameSessionEvent::StartSuccess,
            WordleEvent::WordChecked {
                correct_positions,
                contained_in_word,
                ..
            } => GameSessionEvent::CheckWordResult {
                correct_positions: correct_positions.clone(),
                contained_in_word: contained_in_word.clone(),
            },
        }
    }
}

// 会话状态枚举，定义会话可能的状态
#[derive(Default, Debug, Clone, Encode, Decode, TypeInfo)]
pub enum SessionStatus {
    #[default]
    Init, // 初始化
    WaitUserInput,              // 等待用户输入
    WaitWordleStartReply,       // 等待Wordle开始回复
    WaitWordleCheckWordReply,   // 等待Wordle检查单词回复
    ReplyReceived(WordleEvent), // 收到回复
    GameOver(GameStatus),       // 游戏结束
}

// 会话信息结构体，包含会话的详细信息
#[derive(Default, Debug, Clone, Encode, Decode, TypeInfo)]
pub struct SessionInfo {
    pub session_id: MessageId,            // 会话ID
    pub original_msg_id: MessageId,       // 原始消息ID
    pub send_to_wordle_msg_id: MessageId, // 发送到Wordle的消息ID
    pub tries: u8,                        // 尝试次数
    pub session_status: SessionStatus,    // 会话状态
}

impl SessionInfo {
    // 判断是否处于等待回复状态
    pub fn is_wait_reply_status(&self) -> bool {
        matches!(
            self.session_status,
            SessionStatus::WaitWordleCheckWordReply | SessionStatus::WaitWordleStartReply
        )
    }
}
