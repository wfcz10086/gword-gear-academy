use gsession_io::*;
use gtest::{Log, ProgramBuilder, System};

const GAME_SESSION_PROGRAM_ID: u64 = 1;
const WORDLE_PROGRAM_ID: u64 = 2;
// 用户ID
const USER: u64 = 50;

#[test]
fn test_win() {
    let system = System::new();
    system.init_logger();

    let game_session_program = init_program(
        &system,
        "./target/wasm32-unknown-unknown/gsession.opt.wasm",
        GAME_SESSION_PROGRAM_ID,
    );
    let wordle_program = init_program(
        &system,
        "./target/wasm32-unknown-unknown/gword.opt.wasm",
        WORDLE_PROGRAM_ID,
    );

    // 案例1：初始化wordle_program
    assert_program_init(&wordle_program, USER);

    // 案例2：初始化game_session_program
    assert_program_init(&game_session_program, USER);

    // 案例4：StartGame成功
    assert_start_game_success(&game_session_program, USER);

    // 案例8：CheckWord成功，但未猜中
    assert_check_word_result(&game_session_program, USER, "house", vec![0, 1, 3, 4], vec![]);

    // 新增测试单词
    assert_check_word_fail(&game_session_program, USER, "apple");
    assert_check_word_fail(&game_session_program, USER, "grape");

    // 案例3：CheckWord失败：用户不在游戏中
    assert_check_word_fail(&game_session_program, USER, "abcde");

    // 案例9：CheckWord成功并已猜中
    assert_game_over(&game_session_program, USER, "human", GameStatus::Win);

    // 案例6：CheckWord失败：无效单词
    assert_check_word_fail(&game_session_program, USER, "sssss");

    // 案例5：StartGame失败：用户已在游戏中
    assert!(game_session_program.send(USER, GameSessionAction::StartGame).main_failed());

    // 案例7：CheckWord失败：无效单词
    assert_check_word_fail(&game_session_program, USER, "caonima");

    // 新增测试单词
    assert_check_word_fail(&game_session_program, USER, "and");

    // 案例10：CheckWord失败：用户不在游戏中
    assert_check_word_fail(&game_session_program, 51, "kkkk");

    print_state(&game_session_program);
}

#[test]
fn test_tried_limit() {
    let system = System::new();
    system.init_logger();

    let game_session_program = init_program(
        &system,
        "./target/wasm32-unknown-unknown/game_session.opt.wasm",
        GAME_SESSION_PROGRAM_ID,
    );
    let wordle_program = init_program(
        &system,
        "./target/wasm32-unknown-unknown/wordle.opt.wasm",
        WORDLE_PROGRAM_ID,
    );

    assert_program_init(&wordle_program, USER);
    assert_program_init(&game_session_program, USER);
    assert_start_game_success(&game_session_program, USER);

    for i in 0..5 {
        if i == 4 {
            assert_game_over(&game_session_program, USER, "house", GameStatus::Lose);
        } else {
            assert_check_word_result(&game_session_program, USER, "house", vec![0, 1, 3, 4], vec![]);
        }
    }

    print_state(&game_session_program);
}

#[test]
#[ignore]
fn test_dealyed_logic() {
    let system = System::new();
    system.init_logger();

    let game_session_program = init_program(
        &system,
        "./target/wasm32-unknown-unknown/game_session.opt.wasm",
        GAME_SESSION_PROGRAM_ID,
    );
    let wordle_program = init_program(
        &system,
        "./target/wasm32-unknown-unknown/wordle.opt.wasm",
        WORDLE_PROGRAM_ID,
    );

    assert_program_init(&wordle_program, USER);
    assert_program_init(&game_session_program, USER);
    assert_start_game_success(&game_session_program, USER);

    // 案例4：延迟200个区块（10分钟）
    let result = system.spend_blocks(200);
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::GameOver(GameStatus::Lose));
    assert!(result[0].contains(&log));

    print_state(&game_session_program);
}

// 初始化程序
fn init_program(system: &System, path: &str, program_id: u64) -> ProgramBuilder {
    ProgramBuilder::from_file(path)
        .with_id(program_id)
        .build(system)
}

// 校验程序初始化
fn assert_program_init(program: &ProgramBuilder, user: u64) {
    let res = program.send_bytes(user, []);
    assert!(!res.main_failed());
}

// 校验StartGame成功
fn assert_start_game_success(program: &ProgramBuilder, user: u64) {
    let res = program.send(user, GameSessionAction::StartGame);
    let log = Log::builder()
        .dest(user)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::StartSuccess);
    assert!(!res.main_failed() && res.contains(&log));
}

// 校验CheckWord失败
fn assert_check_word_fail(program: &ProgramBuilder, user: u64, word: &str) {
    let res = program.send(
        user,
        GameSessionAction::CheckWord {
            word: word.to_string(),
        },
    );
    assert!(res.main_failed());
}

// 校验CheckWord结果
fn assert_check_word_result(
    program: &ProgramBuilder,
    user: u64,
    word: &str,
    correct_positions: Vec<usize>,
    contained_in_word: Vec<usize>,
) {
    let res = program.send(
        user,
        GameSessionAction::CheckWord {
            word: word.to_string(),
        },
    );
    let log = Log::builder()
        .dest(user)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::CheckWordResult {
            correct_positions,
            contained_in_word,
        });
    assert!(!res.main_failed() && res.contains(&log));
}

// 校验游戏结束
fn assert_game_over(program: &ProgramBuilder, user: u64, word: &str, status: GameStatus) {
    let res = program.send(
        user,
        GameSessionAction::CheckWord {
            word: word.to_string(),
        },
    );
    let log = Log::builder()
        .dest(user)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::GameOver(status));
    assert!(!res.main_failed() && res.contains(&log));
}

// 打印状态
fn print_state(program: &ProgramBuilder) {
    let state: GameSessionState = program.read_state(b"").unwrap();
    println!("{:?}", state);
}
