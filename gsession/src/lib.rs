#![no_std]
use gsession_io::*;
use gstd::*;

// 尝试次数限制
const TRIES_LIMIT: u8 = 5;

// 静态变量存储游戏会话状态
static mut GAME_SESSION_STATE: Option<GameSession> = None;

// 获取可变的游戏会话状态，确保已初始化
fn get_game_session_mut() -> &'static mut GameSession {
    unsafe {
        GAME_SESSION_STATE
            .as_mut()
            .expect("GAME is not initialized: get_game_session_mut")
    }
}

// 获取不可变的游戏会话状态，确保已初始化
fn get_game_session() -> &'static GameSession {
    unsafe {
        GAME_SESSION_STATE
            .as_ref()
            .expect("GAME is not initialized: get_game_session")
    }
}

#[no_mangle]
extern "C" fn init() {
    // 接收并存储 Wordle 程序地址（在 game_session_io 处理）
    let game_session_init: GameSessionInit =
        msg::load().expect("Unable to decode GameSessionInit: init");
    game_session_init.assert_valid();
    unsafe {
        GAME_SESSION_STATE = Some(game_session_init.into());
    };
}

#[no_mangle]
extern "C" fn handle() {
    let game_session_action: GameSessionAction =
        msg::load().expect("Unable to decode GameSessionAction: handle");
    let game_session = get_game_session_mut();
    match game_session_action {
        // Action 1: 开始游戏
        GameSessionAction::StartGame => {
            let user = msg::source();
            // 检查用户是否已有游戏会话
            let session_info = game_session.sessions.entry(user).or_default();
            match &session_info.session_status {
                SessionStatus::ReplyReceived(wordle_event) => {
                    // 通知用户游戏已成功开始
                    msg::reply::<GameSessionEvent>(wordle_event.into(), 0)
                        .expect("Failed to send reply: StartGame");
                    session_info.session_status = SessionStatus::WaitUserInput;
                }
                SessionStatus::Init
                | SessionStatus::GameOver(..)
                | SessionStatus::WaitWordleStartReply => {
                    // 发送 "StartGame" 消息给 Wordle 程序
                    let send_to_wordle_msg_id = msg::send(
                        game_session.wordle_program_id,
                        WordleAction::StartGame { user },
                        0,
                    )
                    .expect("Failed to send message: StartGame");

                    session_info.session_id = msg::id();
                    session_info.original_msg_id = msg::id();
                    session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
                    session_info.tries = 0;
                    session_info.session_status = SessionStatus::WaitWordleStartReply;
                    // 发送延迟消息以监控游戏进度，延迟为 200 区块（10 分钟）
                    msg::send_delayed(
                        exec::program_id(),
                        GameSessionAction::CheckGameStatus {
                            user,
                            session_id: msg::id(),
                        },
                        0,
                        200,
                    )
                    .expect("Failed to send delayed message: StartGame");
                    // 等待响应
                    exec::wait();
                }
                SessionStatus::WaitUserInput | SessionStatus::WaitWordleCheckWordReply => {
                    panic!("User is already in the game: StartGame");
                }
            }
        }
        // Action 2: 检查单词
        GameSessionAction::CheckWord { word } => {
            let user = msg::source();
            let session_info = game_session.sessions.entry(user).or_default();
            match &session_info.session_status {
                SessionStatus::ReplyReceived(wordle_event) => {
                    // 增加尝试次数
                    session_info.tries += 1;
                    // 检查单词是否猜对
                    if wordle_event.has_guessed() {
                        // 若猜对，切换为游戏结束状态（胜利）
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Win);
                        msg::reply(GameSessionEvent::GameOver(GameStatus::Win), 0)
                            .expect("Failed to send reply: CheckWord");
                    } else if session_info.tries == TRIES_LIMIT {
                        // 若用尽所有尝试，切换为游戏结束状态（失败）
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Lose);
                        msg::reply(GameSessionEvent::GameOver(GameStatus::Lose), 0)
                            .expect("Failed to send reply: CheckWord");
                    } else {
                        msg::reply::<GameSessionEvent>(wordle_event.into(), 0)
                            .expect("Failed to send reply: CheckWord");
                        session_info.session_status = SessionStatus::WaitUserInput;
                    }
                }
                // 确保游戏存在且在正确状态
                SessionStatus::WaitUserInput | SessionStatus::WaitWordleCheckWordReply => {
                    // 验证单词长度为五且为小写
                    assert!(
                        word.len() == 5 && word.chars().all(|c| c.is_lowercase()),
                        "Invalid word: CheckWord"
                    );
                    // 发送 "CheckWord" 消息给 Wordle 程序
                    let send_to_wordle_msg_id = msg::send(
                        game_session.wordle_program_id,
                        WordleAction::CheckWord { user, word },
                        0,
                    )
                    .expect("Failed to send message: CheckWord");
                    session_info.original_msg_id = msg::id();
                    session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
                    session_info.session_status = SessionStatus::WaitWordleCheckWordReply;
                    // 等待回复
                    exec::wait();
                }
                SessionStatus::Init
                | SessionStatus::WaitWordleStartReply
                | SessionStatus::GameOver(..) => {
                    panic!("User is not in the game: CheckWord");
                }
            }
        }
        // Action 3: 检查游戏状态
        GameSessionAction::CheckGameStatus { user, session_id } => {
            if msg::source() == exec::program_id() {
                if let Some(session_info) = game_session.sessions.get_mut(&user) {
                    if session_id == session_info.session_id
                        && !matches!(session_info.session_status, SessionStatus::GameOver(..))
                    {
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Lose);
                        msg::send(user, GameSessionEvent::GameOver(GameStatus::Lose), 0)
                            .expect("Failed to send reply: CheckGameStatus");
                    }
                }
            }
        }
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_to = msg::reply_to().expect("Failed to query reply_to data: handle_reply");
    let wordle_event: WordleEvent =
        msg::load().expect("Unable to decode WordleEvent: handle_reply");
    let game_session = get_game_session_mut();
    let user = wordle_event.get_user();
    if let Some(session_info) = game_session.sessions.get_mut(user) {
        if reply_to == session_info.send_to_wordle_msg_id && session_info.is_wait_reply_status() {
            session_info.session_status = SessionStatus::ReplyReceived(wordle_event);
            exec::wake(session_info.original_msg_id).expect("Failed to wake message: handle_reply");
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let game_session = get_game_session();
    msg::reply::<GameSessionState>(game_session.into(), 0)
        .expect("Failed to encode or reply from state: state");
}
