use raylib_ffi::{*, colors::WHITE};
use std::ffi::c_void;
use tinyrand::*;

fn main() {
    init_gui();

    let mut screen_buffer_data = [ Color{r:0,g:0,b:0,a:0xFF} ; 400*300];
    let screen_buffer = Image {
        data: screen_buffer_data.as_mut_ptr() as *mut c_void,
        width: 400,
        height: 300,
        format: enums::PixelFormat::R8g8b8a8 as i32,
        mipmaps: 1,
    };
    let screen_buffer_texture 
        = load_texture_from_image(screen_buffer);
    let palette = generate_palette();
    let mut fire_buffer = [ 0u8; 400*300 ];
    let mut rng = StdRand::default();

    while ! window_should_close() {
        begin_drawing();

        draw_next_frame(&mut screen_buffer_data, &mut fire_buffer, &palette, &mut rng);

        // Pasar a la GPU
        update_texture(
            screen_buffer_texture, 
            &screen_buffer_data
        );
        draw_texture(screen_buffer_texture, 0, 0, WHITE);
        end_drawing()
    }
    println!("Hello, world!");

    close_window();
}

fn generate_palette() -> [Color; 256] {
    let mut pal = [ Color{r:0,g:0,b:0,a:0xFF} ; 256];

    for i in 0..=84 {
        pal[i].r = (i * (0xFF / 85)) as u8;
    }
    for i in 85..=169 {
        pal[i].r = 255;
        pal[i].g = ((i-85) * (0xFF / 85)) as u8;
    }
    for i in 170..=255 {
        pal[i].r = 255;
        pal[i].g = 255;
        pal[i].b = ((i-170) * (0xFF / 85)) as u8;
    }

    pal
}

fn draw_the_palette(screen: &mut [Color], pal: &[Color]) {
    for i in 0..pal.len() {
        let init = i*400 + 50;
        let pixeles = &mut screen[ init..(init+4) ];
        pixeles[0] = pal[i];
        pixeles[1] = pal[i];
        pixeles[2] = pal[i];
        pixeles[3] = pal[i];
    }
}

fn fill_bottom_with_random_ashes(fire_buf: &mut[u8], rng: &mut Wyrand) {
    let end   = fire_buf.len();
    let start = end - 400;
    for i in start..end {
        fire_buf[i] = rng.next_range(0..256u16) as u8;
    }
}

fn convert_fire_buffer_to_screen(fire_buf: &[u8], pal: &[Color], screen: &mut [Color]) {
    for i in 0..fire_buf.len() {
        let heat = fire_buf[i] as usize;
        screen[i] = pal[ heat ];
    }
}

fn calculate_next_fire_frame(fire_buf: &mut[u8]) {
    let mut old_fire_buf = [0u8; 400*300];
    old_fire_buf.clone_from_slice(fire_buf);

    for y in 0..299 {
        for x in 1..399 {
            let i = (y*400 + x) as usize;

            fire_buf[i] = ((
                    10 * old_fire_buf[i - 1] as u64
                +   20 * old_fire_buf[i + 0] as u64
                +   10 * old_fire_buf[i + 1] as u64
                +  160 * old_fire_buf[i - 1 + 400] as u64
                +  320 * old_fire_buf[i + 0 + 400] as u64
                +  160 * old_fire_buf[i + 1 + 400] as u64
            ) / 680) as u8;
        }
    }
}

fn draw_next_frame(screen: &mut [Color], fire_buf: &mut[u8], pal: &[Color], rng: &mut Wyrand) {
    draw_the_palette(screen, pal);
    fill_bottom_with_random_ashes(fire_buf, rng);
    calculate_next_fire_frame(fire_buf);
    convert_fire_buffer_to_screen(fire_buf, pal, screen);
}

fn update_texture(tex: Texture2D, data: &[Color]) {
    unsafe{ 
        UpdateTexture(tex, data.as_ptr() as *const c_void);
    }
}

fn draw_texture(tex: Texture2D, x: i32, y: i32, tint: Color) {
    unsafe {
        DrawTexture(tex, x, y, tint);
    }
}

fn load_texture_from_image(img: Image) -> Texture2D {
    unsafe {
        LoadTextureFromImage(img)
    }
}

fn init_gui() {
    unsafe {
        InitWindow(400, 300, "Fire".as_ptr() as *const i8);
    }
}

fn window_should_close() -> bool {
    unsafe { WindowShouldClose() }    
}

fn begin_drawing() {
    unsafe {
        BeginDrawing();
    }    
}

fn end_drawing() {
    unsafe {
        EndDrawing();
    }    
}

fn close_window() {
    unsafe {
        CloseWindow();   
    }
}

fn draw_pixel(x: i32, y: i32, color: Color) {
    unsafe {
        DrawPixel(x, y, color)
    }
}