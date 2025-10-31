 use eframe::egui;
use rand::Rng;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;
const TILE_SIZE: f32 = 40.0;
const ANIMATION_SPEED: f32 = 300.0; // 像素/秒

// 不同颜色的宝石用数字表示：1=红，2=绿，3=蓝，4=黄，5=紫
type Board = [[u8; BOARD_WIDTH]; BOARD_HEIGHT];

// 方块动画状态结构体
// 用于存储单个方块在下落动画过程中的所有状态信息
#[derive(Clone)]  // 允许克隆，便于复制动画对象
struct TileAnimation {
    start_row: f32,      // 动画开始时的起始行位置（浮点数，支持像素级精确定位）
    target_row: f32,     // 动画结束时的目标行位置（方块最终要到达的行）
    current_row: f32,    // 当前动画帧中的行位置（在start_row和target_row之间插值）
    col: usize,          // 方块所在的列索引（0到BOARD_WIDTH-1）
    value: u8,           // 方块的值/颜色（1=红，2=绿，3=蓝，4=黄，5=紫）
    is_active: bool,     // 动画是否还在进行中（true=正在动画，false=已完成或未开始）
}

// 游戏主状态结构体
// 存储整个游戏的所有状态数据，包括棋盘、分数、用户交互、动画状态等
struct Game {
    board: Board,                                    // 8x8的游戏棋盘，存储每个位置的宝石颜色值（0=空，1-5=不同颜色）
    score: u32,                                      // 当前得分（累计分数）
    target_score: u32,                               // 目标分数（达到此分数即可获胜）
    selected: Option<(usize, usize)>,                // 当前选中的方块坐标（None=未选中，Some((行, 列))=已选中）
    pending_removal: Vec<(usize, usize)>,            // 待消除的方块坐标列表（用于在消除前高亮显示）
    animation_timer: f32,                            // 动画计时器（秒），用于控制消除高亮显示的时间
    game_over: bool,                                 // 游戏是否结束（true=已结束，false=进行中）
    falling_tiles: Vec<TileAnimation>,               // 正在下落的方块列表（存储所有当前正在播放下落动画的方块）
    is_animating: bool,                              // 是否正在播放动画（true=有动画进行中，false=无动画，可以接受用户输入）
}

impl Game {
    fn new() -> Self {
        let mut game = Game {
            board: [[0; BOARD_WIDTH]; BOARD_HEIGHT],
            score: 0,
            target_score: 2000,
            selected: None,
            pending_removal: Vec::new(),
            animation_timer: 0.0,
            game_over: false,
            falling_tiles: Vec::new(),
            is_animating: false,
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

        // 准备并播放下落动画（基于消除后的棋盘状态，不要在动画前更新棋盘）
        self.prepare_fall_animation();

        true
    }

    // 准备下落动画
    fn prepare_fall_animation(&mut self) {
        self.falling_tiles.clear();
        self.is_animating = true;
        
        for j in 0..BOARD_WIDTH {
            // 计算每个方块需要下落的距离
            let mut fall_distances = vec![0; BOARD_HEIGHT];
            let mut write_pos = BOARD_HEIGHT - 1;
            
            for read_pos in (0..BOARD_HEIGHT).rev() {
                if self.board[read_pos][j] != 0 {
                    let fall_distance = write_pos as i32 - read_pos as i32;
                    if fall_distance > 0 {
                        fall_distances[read_pos] = fall_distance as usize;
                    }
                    if write_pos > 0 {
                        write_pos -= 1;
                    }
                }
            }
            
            // 创建动画对象
            for i in 0..BOARD_HEIGHT {
                if fall_distances[i] > 0 && self.board[i][j] != 0 {
                    self.falling_tiles.push(TileAnimation {
                        start_row: i as f32,
                        target_row: (i + fall_distances[i]) as f32,
                        current_row: i as f32,
                        col: j,
                        value: self.board[i][j],
                        is_active: true,
                    });
                }
            }
        }
        
        // 如果没有创建任何动画对象，说明不需要动画
        if self.falling_tiles.is_empty() {
            self.is_animating = false;
        }
    }

    // 带动画的方块下落
    fn drop_tiles_with_animation(&mut self) {
        // 在动画过程中，更新实际棋盘
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
    
    // 更新下落动画
    fn update_fall_animation(&mut self, delta_time: f32) -> bool {
        if !self.is_animating {
            return false;
        }
        if self.falling_tiles.is_empty() {
            // 如果没有动画对象但标记为正在动画，清除标记
            self.is_animating = false;
            return false;
        }
        
        let mut all_finished = true;
        let pixel_delta = ANIMATION_SPEED * delta_time / TILE_SIZE;
        
        for tile in &mut self.falling_tiles {
            if tile.is_active && tile.current_row < tile.target_row {
                tile.current_row += pixel_delta;
                if tile.current_row >= tile.target_row {
                    tile.current_row = tile.target_row;
                    tile.is_active = false;
                } else {
                    all_finished = false;
                }
            }
        }
        
        if all_finished {
            // 动画完成后，更新棋盘状态（把方块移到正确位置）
            self.drop_tiles_with_animation();
            
            // 检查是否有新的匹配需要消除
            let has_matches = !self.find_matches().is_empty();
            
            if has_matches {
                // 有新的匹配，清除动画状态，重置计时器以便立即触发下一轮消除
                self.falling_tiles.clear();
                self.is_animating = false;
                self.animation_timer = 0.6; // 设置为大于0.5，让update函数立即触发消除
            } else {
                // 没有新匹配，填充顶部空白位置的新方块
                self.fill_empty();
                // 让新方块下落（更新棋盘状态）
                self.drop_tiles_with_animation();
                // 检查新方块是否需要下落动画
                self.prepare_fall_animation();
                // 如果 prepare_fall_animation 没有创建任何动画（因为新方块已经在正确位置），则清除动画状态
                if self.falling_tiles.is_empty() {
                    self.is_animating = false;
                }
            }
        }
        
        !all_finished
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
                    
                    let temp_game = Game { 
                        board: test_board, 
                        score: 0, 
                        target_score: 0, 
                        selected: None, 
                        pending_removal: Vec::new(), 
                        animation_timer: 0.0, 
                        game_over: false,
                        falling_tiles: Vec::new(),
                        is_animating: false,
                    };
                    if temp_game.find_matches().len() > 0 {
                        return true;
                    }
                }
                if i < BOARD_HEIGHT - 1 {
                    let mut test_board = self.board;
                    test_board[i][j] = self.board[i + 1][j];
                    test_board[i + 1][j] = self.board[i][j];
                    
                    let temp_game = Game { 
                        board: test_board, 
                        score: 0, 
                        target_score: 0, 
                        selected: None, 
                        pending_removal: Vec::new(), 
                        animation_timer: 0.0, 
                        game_over: false,
                        falling_tiles: Vec::new(),
                        is_animating: false,
                    };
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
        let delta_time = ctx.input(|i| i.unstable_dt);
        self.animation_timer += delta_time;

        // 更新下落动画
        if self.update_fall_animation(delta_time) {
            // 动画进行中，不进行其他操作
            return;
        }

        // 检查是否有空格需要处理（如果不在动画中且有空格）
        if !self.is_animating && self.pending_removal.is_empty() {
            let mut has_empty = false;
            for i in 0..BOARD_HEIGHT {
                for j in 0..BOARD_WIDTH {
                    if self.board[i][j] == 0 {
                        has_empty = true;
                        break;
                    }
                }
                if has_empty {
                    break;
                }
            }
            
            if has_empty {
                // 有空格，让方块下落并准备动画
                self.drop_tiles_with_animation();
                self.fill_empty();
                self.drop_tiles_with_animation();
                self.prepare_fall_animation();
                // 如果仍然没有动画，说明不需要动画，但应该继续
                if self.falling_tiles.is_empty() {
                    self.is_animating = false;
                }
                return; // 等待下一帧继续
            }
        }

        // 清除待消除标记
        if self.animation_timer > 0.3 && !self.pending_removal.is_empty() {
            self.pending_removal.clear();
            self.animation_timer = 0.0;
        }

        // 自动消除（仅在动画结束后）
        if self.pending_removal.is_empty() && !self.is_animating && self.animation_timer > 0.5 {
            while self.remove_matches() {
                self.animation_timer = 0.0;
                break;
            }
        }

        // 检查是否有可用移动
        if self.pending_removal.is_empty() && !self.has_moves() {
            if !self.game_over {
                // 没有可用移动时，先尝试重新洗牌
                let attempts = 5;
                let mut shuffled = false;
                for _ in 0..attempts {
                    self.fill_board();
                    if self.has_moves() {
                        shuffled = true;
                        break;
                    }
                }
                // 如果重新洗牌后还是没有可用移动，游戏结束
                if !shuffled {
                    self.game_over = true;
                }
            }
        }
        
        // 检查是否达到目标分数
        if self.score >= self.target_score && !self.game_over {
            self.game_over = true;
        }

        // 清除待消除标记
        if self.animation_timer > 0.3 && !self.pending_removal.is_empty() {
            self.pending_removal.clear();
            self.animation_timer = 0.0;
        }

        // 如果正在播放动画，持续请求重绘
        if self.is_animating {
            ctx.request_repaint();
        }
    }
}

impl eframe::App for Game {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("三消游戏");
                ui.label(format!("分数: {} / {}", self.score, self.target_score));
                
                // 检查游戏结束
                if self.game_over || self.score >= self.target_score {
                    ui.add_space(10.0);
                    ui.heading(if self.score >= self.target_score {
                        "恭喜过关！"
                    } else {
                        "游戏结束"
                    });
                    if ui.button("重新开始").clicked() {
                        *self = Game::new();
                    }
                    return;
                }
                
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

                        // 检查是否被点击（动画期间不能点击）
                        if !self.is_animating && response.clicked() {
                            if let Some(click_pos) = response.interact_pointer_pos() {
                                if tile_rect.contains(click_pos) {
                                    self.handle_click(i, j);
                                }
                            }
                        }
                    }
                }

                // 首先绘制固定位置的方块（非动画中的）
                for i in 0..BOARD_HEIGHT {
                    for j in 0..BOARD_WIDTH {
                        // 检查这个位置是否有正在动画的方块
                        let has_falling = self.falling_tiles.iter()
                            .any(|t| t.col == j && (t.start_row as usize) == i);
                        
                        if has_falling {
                            continue; // 这个位置的方块正在动画，稍后绘制
                        }
                        
                        let x = start_x + j as f32 * TILE_SIZE;
                        let y = start_y + i as f32 * TILE_SIZE;
                        
                        let tile_rect = egui::Rect::from_min_size(
                            egui::Pos2::new(x, y),
                            egui::Vec2::new(TILE_SIZE - 2.0, TILE_SIZE - 2.0),
                        );

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
                
                // 绘制正在下落的方块（覆盖在上方）
                for tile in &self.falling_tiles {
                    if !tile.is_active {
                        continue;
                    }
                    
                    let x = start_x + tile.col as f32 * TILE_SIZE;
                    let y = start_y + tile.current_row * TILE_SIZE;
                    
                    let tile_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(x, y),
                        egui::Vec2::new(TILE_SIZE - 2.0, TILE_SIZE - 2.0),
                    );
                    
                    let color = Self::get_color(tile.value);
                    painter.rect_filled(tile_rect, 2.0, color);
                    painter.rect_stroke(tile_rect, 2.0, (1.0, egui::Color32::from_rgb(150, 150, 150)));
                }

                ui.add_space(20.0);
                ui.label("操作说明：点击相邻的两个方块来交换");
                ui.label(format!("目标：达到 {} 分", self.target_score));
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 550.0])
            .with_title("三消游戏"),
        ..Default::default()
    };
    
    eframe::run_native(
        "三消游戏",
        options,
        Box::new(|cc| {
            // 配置中文字体
            let mut fonts = egui::FontDefinitions::default();
            
            // 尝试使用系统字体
            #[cfg(target_os = "windows")]
            {
                // Windows 系统中文字体路径
                if let Ok(font_data) = std::fs::read("C:/Windows/Fonts/msyh.ttc") {
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    fonts.families.get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .insert(0, "chinese".to_owned());
                    fonts.families.get_mut(&egui::FontFamily::Monospace)
                        .unwrap()
                        .push("chinese".to_owned());
                } else if let Ok(font_data) = std::fs::read("C:/Windows/Fonts/simsun.ttc") {
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    fonts.families.get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .insert(0, "chinese".to_owned());
                    fonts.families.get_mut(&egui::FontFamily::Monospace)
                        .unwrap()
                        .push("chinese".to_owned());
                }
            }
            
            cc.egui_ctx.set_fonts(fonts);
            
            Box::new(Game::new())
        }),
    )
}
