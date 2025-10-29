use std::io::{self, Write};
use rand::Rng;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

// 不同颜色的宝石用数字表示：1=红，2=绿，3=蓝，4=黄，5=紫
type Board = [[u8; BOARD_WIDTH]; BOARD_HEIGHT];

struct Game {
    board: Board,
    score: u32,
}

impl Game {
    fn new() -> Self {
        let mut game = Game {
            board: [[0; BOARD_WIDTH]; BOARD_HEIGHT],
            score: 0,
        };
        game.fill_board();
        // 确保初始状态没有三消
        while game.find_matches().len() > 0 {
            game.fill_board();
        }
        game
    }

    // 用随机颜色填充游戏板
    fn fill_board(&mut self) {
        let mut rng = rand::thread_rng();
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                self.board[i][j] = rng.gen_range(1..=5);
            }
        }
    }

    // 打印游戏板
    fn print_board(&self) {
        print!("{}[2J{}[H", 27 as char, 27 as char); // 清屏
        println!("=== 三消游戏 === 分数: {}\n", self.score);
        print!("   ");
        for j in 0..BOARD_WIDTH {
            print!("{} ", j);
        }
        println!();
        
        for (i, row) in self.board.iter().enumerate() {
            print!("{}  ", i);
            for &cell in row.iter() {
                let symbol = match cell {
                    1 => "🔴",
                    2 => "🟢",
                    3 => "🔵",
                    4 => "🟡",
                    5 => "🟣",
                    _ => "⚪",
                };
                print!("{} ", symbol);
            }
            println!();
        }
        println!("\n操作说明：");
        println!("输入格式：行 列 (例如: 0 1 表示选择第0行第1列)");
        println!("先选择第一个方块，再选择相邻的第二个方块来交换");
        println!("输入 'q' 退出游戏\n");
    }

    // 查找所有可以消除的匹配（三个或更多连续相同）
    fn find_matches(&self) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();
        let mut marked = [[false; BOARD_WIDTH]; BOARD_HEIGHT];

        // 检查水平匹配
        for i in 0..BOARD_HEIGHT {
            let mut count = 1;
            let mut start = 0;
            for j in 1..BOARD_WIDTH {
                if self.board[i][j] == self.board[i][j - 1] && self.board[i][j] != 0 {
                    count += 1;
                } else {
                    if count >= 3 {
                        for k in start..j {
                            marked[i][k] = true;
                        }
                    }
                    count = 1;
                    start = j;
                }
            }
            if count >= 3 {
                for k in start..BOARD_WIDTH {
                    marked[i][k] = true;
                }
            }
        }

        // 检查垂直匹配
        for j in 0..BOARD_WIDTH {
            let mut count = 1;
            let mut start = 0;
            for i in 1..BOARD_HEIGHT {
                if self.board[i][j] == self.board[i - 1][j] && self.board[i][j] != 0 {
                    count += 1;
                } else {
                    if count >= 3 {
                        for k in start..i {
                            marked[k][j] = true;
                        }
                    }
                    count = 1;
                    start = i;
                }
            }
            if count >= 3 {
                for k in start..BOARD_HEIGHT {
                    marked[k][j] = true;
                }
            }
        }

        // 收集所有标记的位置
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                if marked[i][j] {
                    matches.push((i, j));
                }
            }
        }

        matches
    }

    // 消除匹配的方块并让上方方块下落
    fn remove_matches(&mut self) -> bool {
        let matches = self.find_matches();
        if matches.is_empty() {
            return false;
        }

        // 计算分数：3个=100分，4个=200分，5个及以上=300分
        let match_count = matches.len();
        if match_count >= 5 {
            self.score += 300;
        } else if match_count == 4 {
            self.score += 200;
        } else {
            self.score += 100;
        }

        // 消除匹配的方块（设为0）
        for (i, j) in &matches {
            self.board[*i][*j] = 0;
        }

        // 让方块下落
        self.drop_tiles();

        // 填充空白位置
        self.fill_empty();

        true
    }

    // 让方块下落
    fn drop_tiles(&mut self) {
        for j in 0..BOARD_WIDTH {
            let mut write_pos = BOARD_HEIGHT - 1;
            for read_pos in (0..BOARD_HEIGHT).rev() {
                if self.board[read_pos][j] != 0 {
                    if read_pos != write_pos {
                        self.board[write_pos][j] = self.board[read_pos][j];
                        self.board[read_pos][j] = 0;
                    }
                    if write_pos > 0 {
                        write_pos -= 1;
                    }
                }
            }
        }
    }

    // 填充空白位置
    fn fill_empty(&mut self) {
        let mut rng = rand::thread_rng();
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                if self.board[i][j] == 0 {
                    self.board[i][j] = rng.gen_range(1..=5);
                }
            }
        }
    }

    // 交换两个相邻的方块
    fn swap(&mut self, row1: usize, col1: usize, row2: usize, col2: usize) -> bool {
        // 检查是否相邻
        let row_diff = (row1 as i32 - row2 as i32).abs();
        let col_diff = (col1 as i32 - col2 as i32).abs();
        
        if row_diff + col_diff != 1 {
            return false;
        }

        // 交换
        let temp = self.board[row1][col1];
        self.board[row1][col1] = self.board[row2][col2];
        self.board[row2][col2] = temp;

        true
    }

    // 检查是否有可用的移动
    fn has_moves(&self) -> bool {
        // 检查水平和垂直相邻的方块
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                // 检查右邻居
                if j < BOARD_WIDTH - 1 {
                    let mut test_board = self.board;
                    test_board[i][j] = self.board[i][j + 1];
                    test_board[i][j + 1] = self.board[i][j];
                    
                    // 临时创建游戏来检查匹配
                    let temp_game = Game { board: test_board, score: 0 };
                    if temp_game.find_matches().len() > 0 {
                        return true;
                    }
                }
                // 检查下邻居
                if i < BOARD_HEIGHT - 1 {
                    let mut test_board = self.board;
                    test_board[i][j] = self.board[i + 1][j];
                    test_board[i + 1][j] = self.board[i][j];
                    
                    let temp_game = Game { board: test_board, score: 0 };
                    if temp_game.find_matches().len() > 0 {
                        return true;
                    }
                }
            }
        }
        false
    }

    // 游戏主循环
    fn play(&mut self) {
        loop {
            self.print_board();

            // 如果当前状态有自动三消，先消除
            while self.remove_matches() {
                std::thread::sleep(std::time::Duration::from_millis(500));
                self.print_board();
            }

            // 检查是否还有可用的移动
            if !self.has_moves() {
                println!("没有可用的移动！重新洗牌...");
                self.fill_board();
                continue;
            }

            // 获取用户输入
            print!("选择第一个方块 (行 列): ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            if input.trim() == "q" {
                println!("游戏结束！最终分数: {}", self.score);
                break;
            }

            let coords1: Vec<&str> = input.trim().split_whitespace().collect();
            if coords1.len() != 2 {
                println!("输入格式错误！请使用：行 列");
                continue;
            }

            let row1: usize = match coords1[0].parse() {
                Ok(n) => n,
                Err(_) => {
                    println!("无效的行号！");
                    continue;
                }
            };

            let col1: usize = match coords1[1].parse() {
                Ok(n) => n,
                Err(_) => {
                    println!("无效的列号！");
                    continue;
                }
            };

            if row1 >= BOARD_HEIGHT || col1 >= BOARD_WIDTH {
                println!("坐标超出范围！行: 0-{}, 列: 0-{}", BOARD_HEIGHT - 1, BOARD_WIDTH - 1);
                continue;
            }

            print!("选择第二个方块 (行 列): ");
            io::stdout().flush().unwrap();
            let mut input2 = String::new();
            io::stdin().read_line(&mut input2).unwrap();

            let coords2: Vec<&str> = input2.trim().split_whitespace().collect();
            if coords2.len() != 2 {
                println!("输入格式错误！请使用：行 列");
                continue;
            }

            let row2: usize = match coords2[0].parse() {
                Ok(n) => n,
                Err(_) => {
                    println!("无效的行号！");
                    continue;
                }
            };

            let col2: usize = match coords2[1].parse() {
                Ok(n) => n,
                Err(_) => {
                    println!("无效的列号！");
                    continue;
                }
            };

            if row2 >= BOARD_HEIGHT || col2 >= BOARD_WIDTH {
                println!("坐标超出范围！行: 0-{}, 列: 0-{}", BOARD_HEIGHT - 1, BOARD_WIDTH - 1);
                continue;
            }

            // 交换方块
            if !self.swap(row1, col1, row2, col2) {
                println!("这两个方块不相邻！只能交换相邻的方块。");
                continue;
            }

            // 检查交换后是否有匹配
            let matches = self.find_matches();
            if matches.is_empty() {
                // 没有匹配，交换回来
                self.swap(row1, col1, row2, col2);
                println!("交换后没有形成三消！已自动撤销。");
                std::thread::sleep(std::time::Duration::from_millis(1000));
            } else {
                // 有匹配，继续消除
                self.remove_matches();
            }
        }
    }
}

fn main() {
    let mut game = Game::new();
    game.play();
}
