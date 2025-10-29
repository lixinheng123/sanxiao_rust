use eframe::egui;
use rand::Rng;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;
const TILE_SIZE: f32 = 40.0;

// 不同颜色的宝石用数字表示：1=红，2=绿，3=蓝，4=黄，5=紫
type Board = [[u8; BOARD_WIDTH]; BOARD_HEIGHT];

struct Game {
    board: Board,
    score: u32,
    selected: Option<(usize, usize)>,
    pending_removal: Vec<(usize, usize)>,
    animation_timer: f32,
}

impl Game {
    fn new() -> Self {
        let mut game = Game {
            board: [[0; BOARD_WIDTH]; BOARD_HEIGHT],
            score: 0,
            selected: None,
            pending_removal: Vec::new(),
            animation_timer: 0.0,
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

    // 获取颜色对应的 RGB
    fn get_color(cell: u8) -> egui::Color32 {
        match cell {
            1 => egui::Color32::from_rgb(255, 80, 80),   // 红
            2 => egui::Color32::from_rgb(80, 255, 80),   // 绿
            3 => egui::Color32::from_rgb(80, 80, 255),   // 蓝
            4 => egui::Color32::from_rgb(255, 255, 80),  // 黄
            5 => egui::Color32::from_rgb(255, 80, 255),  // 紫
            _ => egui::Color32::from_rgb(200, 200, 200), // 灰
        }
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

        // 计算分数
        let match_count = matches.len();
        if match_count >= 5 {
            self.score += 300;
        } else if match_count == 4 {
            self.score += 200;
        } else {
            self.score += 100;
        }

        // 记录要消除的方块
        self.pending_removal = matches.clone();

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
        let row_diff = (row1 as i32 - row2 as i32).abs();
        let col_diff = (col1 as i32 - col2 as i32).abs();
        
        if row_diff + col_diff != 1 {
            return false;
        }

        let temp = self.board[row1][col1];
        self.board[row1][col1] = self.board[row2][col2];
        self.board[row2][col2] = temp;

        true
    }

    // 检查是否有可用的移动
    fn has_moves(&self) -> bool {
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                if j < BOARD_WIDTH - 1 {
                    let mut test_board = self.board;
                    test_board[i][j] = self.board[i][j + 1];
                    test_board[i][j + 1] = self.board[i][j];
                    
                    let temp_game = Game { board: test_board, score: 0, selected: None, pending_removal: Vec::new(), animation_timer: 0.0 };
                    if temp_game.find_matches().len() > 0 {
                        return true;
                    }
                }
                if i < BOARD_HEIGHT - 1 {
                    let mut test_board = self.board;
                    test_board[i][j] = self.board[i + 1][j];
                    test_board[i + 1][j] = self.board[i][j];
                    
                    let temp_game = Game { board: test_board, score: 0, selected: None, pending_removal: Vec::new(), animation_timer: 0.0 };
                    if temp_game.find_matches().len() > 0 {
                        return true;
                    }
                }
            }
        }
        false
    }

    // 处理方块点击
    fn handle_click(&mut self, row: usize, col: usize) {
        if let Some((sel_row, sel_col)) = self.selected {
            if sel_row == row && sel_col == col {
                // 取消选择
                self.selected = None;
            } else if (sel_row == row && (sel_col as i32 - col as i32).abs() == 1)
                || (sel_col == col && (sel_row as i32 - row as i32).abs() == 1)
            {
                // 尝试交换
                if self.swap(sel_row, sel_col, row, col) {
                    let matches = self.find_matches();
                    if matches.is_empty() {
                        // 没有匹配，交换回来
                        self.swap(sel_row, sel_col, row, col);
                    } else {
                        // 有匹配，消除
                        self.remove_matches();
                    }
                }
                self.selected = None;
            } else {
                // 选择新方块
                self.selected = Some((row, col));
            }
        } else {
            // 选择方块
            self.selected = Some((row, col));
        }
    }

    // 更新游戏状态
    fn update(&mut self, ctx: &egui::Context) {
        self.animation_timer += ctx.input().unstable_dt;

        // 自动消除
        if self.pending_removal.is_empty() && self.animation_timer > 0.5 {
            while self.remove_matches() {
                self.animation_timer = 0.0;
                break;
            }
        }

        // 检查是否有可用移动
        if self.pending_removal.is_empty() && !self.has_moves() {
            self.fill_board();
            while self.find_matches().len() > 0 {
                self.fill_board();
            }
        }

        // 清除待消除标记
        if self.animation_timer > 0.3 && !self.pending_removal.is_empty() {
            self.pending_removal.clear();
            self.animation_timer = 0.0;
        }

        ctx.request_repaint();
    }
}

impl eframe::App for Game {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("三消游戏");
                ui.label(format!("分数: {}", self.score));
                ui.add_space(20.0);

                // 绘制游戏板
                let board_size = TILE_SIZE * BOARD_WIDTH as f32;
                let (response, painter) = ui.allocate_painter(
                    egui::Vec2::new(board_size + 20.0, board_size + 20.0),
                    egui::Sense::click(),
                );

                let rect = response.rect;
                let start_x = rect.left() + 10.0;
                let start_y = rect.top() + 10.0;

                // 绘制网格和方块
                for i in 0..BOARD_HEIGHT {
                    for j in 0..BOARD_WIDTH {
                        let x = start_x + j as f32 * TILE_SIZE;
                        let y = start_y + i as f32 * TILE_SIZE;
                        
                        let tile_rect = egui::Rect::from_min_size(
                            egui::Pos2::new(x, y),
                            egui::Vec2::new(TILE_SIZE - 2.0, TILE_SIZE - 2.0),
                        );

                        // 检查是否被点击
                        if response.clicked() {
                            let click_pos = response.interact_pointer_pos().unwrap();
                            if tile_rect.contains(click_pos) {
                                self.handle_click(i, j);
                            }
                        }

                        // 绘制方块背景
                        let mut color = Self::get_color(self.board[i][j]);
                        
                        // 如果被选中，改变颜色
                        if let Some((sel_row, sel_col)) = self.selected {
                            if sel_row == i && sel_col == j {
                                color = color.gamma_multiply(1.5);
                            }
                        }

                        // 如果待消除，变暗
                        if self.pending_removal.contains(&(i, j)) {
                            color = color.gamma_multiply(0.3);
                        }

                        painter.rect_filled(tile_rect, 2.0, color);
                        
                        // 绘制边框
                        let border_color = if let Some((sel_row, sel_col)) = self.selected {
                            if sel_row == i && sel_col == j {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_rgb(150, 150, 150)
                            }
                        } else {
                            egui::Color32::from_rgb(150, 150, 150)
                        };
                        painter.rect_stroke(tile_rect, 2.0, (1.0, border_color));
                    }
                }

                ui.add_space(20.0);
                ui.label("操作说明：点击相邻的两个方块来交换");
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_title("三消游戏"),
        ..Default::default()
    };
    
    eframe::run_native(
        "三消游戏",
        options,
        Box::new(|_cc| Box::new(Game::new())),
    )
}
