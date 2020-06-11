
const NEST_LVL: usize = 6;

#[derive(Clone, Copy)]
struct JsonFormatterLvl {
    is_obj_line_feed: bool,
    is_arr_line_feed: bool,
    ind_char_num: u8,
    ind_char: char,
}

/// 当前支持6级嵌套格式化设置, 从0开始计数. 超过6级嵌套, 则后续嵌套按照第6级的设置格式化;  
#[derive(Copy, Clone)]
pub struct JsonFormatter {
    lvl: [JsonFormatterLvl; NEST_LVL],
}

impl JsonFormatter {
    pub fn new() -> JsonFormatter {
        let mut f = JsonFormatter {lvl: [JsonFormatterLvl{
            is_obj_line_feed: true, is_arr_line_feed: true, ind_char_num: 2, ind_char: ' ',
        }; NEST_LVL]};
        for i in 0..NEST_LVL {
            if i > 0 {
                f.lvl[i].ind_char_num = 1;
                f.lvl[i].is_obj_line_feed = true;
                f.lvl[i].is_arr_line_feed = true;
                f.lvl[i].ind_char = ' ';
            }
        }
        f
    }
    
    pub fn is_obj_line_feed(&self, lvl: usize) -> bool {
        if lvl < NEST_LVL {
            self.lvl[lvl].is_obj_line_feed
        } else {
            self.lvl[NEST_LVL-1].is_obj_line_feed
        }
    }
    
    pub fn is_arr_line_feed(&self, lvl: usize) -> bool {
        if lvl < NEST_LVL {
            self.lvl[lvl].is_arr_line_feed
        } else {
            self.lvl[NEST_LVL - 1].is_arr_line_feed
        }
    }
    
    pub fn ind_char_num(&self, lvl: usize) -> usize {
        if lvl < NEST_LVL {
            self.lvl[lvl].ind_char_num as usize
        } else {
            self.lvl[NEST_LVL-1].ind_char_num as usize
        }
    }
    
    pub fn set_obj_line_feed(&mut self, lvl: usize, is_obj_line_feed: bool) -> &mut Self {
        if lvl < NEST_LVL {
            self.lvl[lvl].is_obj_line_feed = is_obj_line_feed;
        } else {
            self.lvl[NEST_LVL-1].is_obj_line_feed = is_obj_line_feed;
        }
        
        self
    }
    
    pub fn set_ojb_line_feed_all(&mut self, is_obj_line_feed: bool) -> &mut Self {
        for lvl in 0..NEST_LVL {
            self.set_obj_line_feed(lvl, is_obj_line_feed);
        }
        self
    }
    
    pub fn set_arr_line_feed(&mut self, lvl: usize, is_arr_line_feed: bool) -> &mut Self {
        if lvl < NEST_LVL {
            self.lvl[lvl].is_arr_line_feed = is_arr_line_feed;
        } else {
            self.lvl[NEST_LVL-1].is_arr_line_feed = is_arr_line_feed;
        }
        self
    }
    
    pub fn set_arr_line_feed_all(&mut self, is_arr_line_feed: bool) -> &mut Self {
        for lvl in 0..NEST_LVL {
            self.set_arr_line_feed(lvl, is_arr_line_feed);
        }
        self
    }
    
    pub fn set_ind_char_num(&mut self, lvl: usize, ind_char_num: u8) -> &mut Self {
        if lvl < NEST_LVL {
            self.lvl[lvl].ind_char_num = ind_char_num;
        } else {
            self.lvl[NEST_LVL - 1].ind_char_num = ind_char_num;
        }
        self
    }
    
    pub fn set_ind_char_num_all(&mut self, ind_char_num: u8) -> &mut Self {
        for lvl in 0..NEST_LVL {
            self.set_ind_char_num(lvl, ind_char_num);
        }
        self
    }
    
    pub fn ind_char(&self, lvl: usize) -> char {
        if lvl < NEST_LVL {
            self.lvl[lvl].ind_char
        } else {
            self.lvl[NEST_LVL-1].ind_char
        }
    }
    
    pub fn set_ind_char(&mut self, lvl: usize, ind_char: char) -> &mut Self {
        if lvl < NEST_LVL {
            self.lvl[lvl].ind_char = ind_char;
        } else {
            self.lvl[NEST_LVL-1].ind_char = ind_char;
        }
        self
    }
    
    pub fn set_ind_char_all(&mut self, ind_char: char) -> &mut Self {
        for lvl in 0..NEST_LVL {
            self.set_ind_char(lvl, ind_char);
        }
        self
    }
}

