///////////////////////////
// Use directives
use raylib_ffi::*;
use raylib_ffi::colors::*;
use std::ffi::c_void;
use tinyrand::*;

///////////////////////////
// Constants 
const WIDTH : usize = 400;
const HEIGHT: usize = 300;
const SIZE:   usize = WIDTH * HEIGHT;
const PALSIZE:usize = 256;

///////////////////////////
// Comfortable type aliases
type  Palette  = [Color; PALSIZE];
type  ScrBuff  = [Color; SIZE];
type  HeatBuff = [u8; SIZE];
type  AppData  = (ScrBuff, Image, Texture2D, Palette, HeatBuff, Wyrand);

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// MAIN: Entry point of the application
/// 
fn main() {
    // Initialize GUI and Required data
    init_gui();
    let ( mut screen_buffer_data
        , _
        , screen_buffer_texture
        , palette
        , mut fire_buffer
        , mut rng
    ) = init_data();

    // Main Loop until window is required to close
    while ! window_should_close() {
        begin_drawing();

        // Draws the next frame
        draw_next_frame(
              &mut screen_buffer_data
            , &mut fire_buffer
            , &palette
            , &mut rng
        );

        // Updates GPU texture
        update_texture(
            screen_buffer_texture, 
            &screen_buffer_data
        );
        draw_texture(screen_buffer_texture, 0, 0, WHITE);
        
        end_drawing()
    }

    // Cleanup GPU texture and close window
    unload_texture(screen_buffer_texture);
    close_window();
}

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// Initialize all data needed by the application and return it
/// as a tuple with all the elements
/// 
fn init_data() -> AppData {
    // Screen Buffer Data: Our CPU buffer with all the pixels (WIDTH x HEIGHT)
    let mut screen_buffer_data: ScrBuff = [ Color{r:0,g:0,b:0,a:0xFF} ; SIZE];
    
    // Screen Buffer: The Raylib Image Object that points to our Screen Buffer Data
    let screen_buffer = Image {
        data:       screen_buffer_data.as_mut_ptr() as *mut c_void,
        width:      WIDTH  as i32,
        height:     HEIGHT as i32,
        format:     enums::PixelFormat::R8g8b8a8 as i32,
        mipmaps:    1,
    };

    // Screen Buffer Texture: The GPU buffer (texture) that will copy 
    //    all of our CPU data each frame (from screen buffer data)
    let screen_buffer_texture 
        = load_texture_from_image(screen_buffer);
    
    // Palette: Our palette for converting Heat values [0-255] to Colors
    let palette: Palette = generate_palette();

    // Fire Buffer: Our CPU screen buffer data, but in HEAT values. This is
    //    the one we use for all fire calculations. However, before drawing
    //    it, we will have to convert it to Colors using the palette
    let fire_buffer: HeatBuff = [ 0u8; SIZE ];

    // rng: Our tiny random number generator
    let rng = StdRand::default();

    // We return everything as a tuple, giving ownership to the caller
    ( screen_buffer_data, screen_buffer
    , screen_buffer_texture
    , palette, fire_buffer, rng
    )
}

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// Create a palette as a gradient from Black to Red, Red to Yellow and
/// Yellow to White, mimicking the increase in heat
/// 
fn generate_palette() -> Palette {
    let mut pal: Palette = [ Color{r:0,g:0,b:0,a:0xFF} ; PALSIZE ];

    // Black to Red
    for i in 0..=84 {
        pal[i].r = (i * (0xFF / 85)) as u8;
    }
    
    // Red to Yellow
    for i in 85..=169 {
        pal[i].r = 255;
        pal[i].g = ((i-85) * (0xFF / 85)) as u8;
    }

    // Yellow to White
    for i in 170..=255 {
        pal[i].r = 255;
        pal[i].g = 255;
        pal[i].b = ((i-170) * (0xFF / 85)) as u8;
    }

    // Return the palette
    pal
}

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// Draw the palette to the screen buffer (CPU) as a 4-pixel-wide
/// vertical line
/// 
fn draw_the_palette(screen: &mut [Color], pal: &[Color]) {
    for y in 0..pal.len() {
        let init = y*WIDTH + 50;    // Coordinates (50,y)
        let pixels = &mut screen[ init..(init+4) ];
        pixels[0] = pal[y];
        pixels[1] = pal[y];
        pixels[2] = pal[y];
        pixels[3] = pal[y];
    }
}

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// Fill the bottom of the fire_buffer with the ashes. The ashes are no
/// more than random heat values in the bottom of the screen.
///
fn fill_bottom_with_random_ashes(fire_buf: &mut [u8], rng: &mut Wyrand) {
    let end   = fire_buf.len(); // Last pixel at the end of the buffer
    let start = end - WIDTH;    // 1 row earlier, first pixel
    for x in start..end {
        fire_buf[x] = rng.next_range(0..PALSIZE) as u8;
    }
}

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// Convert the fire_buffer (heat values) to Screen Buffer (Colors in our
/// CPU Image). We use the palette for this conversion. Each HEAT value 
/// corresponds to a color, which is defined in the palette.
///
fn convert_fire_buffer_to_screen(fire_buf: &[u8], pal: &[Color], screen: &mut [Color]) {
    for i in 0..fire_buf.len() {
        let heat = fire_buf[i] as usize;    // Get next heat value
        screen[i] = pal[ heat ];                   // Convert it to Color value
    }
}

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// Apply the convolution filter (the fire effect) to the whole fire buffer
/// to calculate the next frame of the fire effect. The convolution effect
/// is a weighted matrix. For each pixel we sum up weighted heat values from
/// influencing surrounding pixels, then we average all of them. The fire
/// dissipates because the average is calculated using integer arithmetics,
/// which makes the calculation lose decimal digits, losing heat.
///
fn calculate_next_fire_frame(fire_buf: &mut[u8]) {
    // Create a copy of the fire buffer before modifying it
    let mut old_fire_buf = [0u8; SIZE];
    old_fire_buf.clone_from_slice(fire_buf);

    // For all pixels in the fire_buffer, top to bottom
    for y in 0..299 {
        for x in 1..399 {
            let i = (y*WIDTH + x) as usize;   // i = index from (x,y) coords

            // This Pixel = weighted average of surrounding pixels
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

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// Performs all the operations required to draw the next frame in the CPU 
/// image, using the previous frame as input
///
fn draw_next_frame(screen: &mut [Color], fire_buf: &mut[u8], pal: &[Color], rng: &mut Wyrand) {
    // Just for visual debugging purposes
    draw_the_palette(screen, pal);

    // Actual steps to calculate next frame
    fill_bottom_with_random_ashes(fire_buf, rng);
    calculate_next_fire_frame(fire_buf);
    convert_fire_buffer_to_screen(fire_buf, pal, screen);
}


///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// 
/// FUNCTIONS TO WRAP Raylib FFI Interface
/// 
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////

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

#[allow(dead_code)] // Don't warn for this function being unused
fn draw_pixel(x: i32, y: i32, color: Color) {
    unsafe {
        DrawPixel(x, y, color)
    }
}

fn unload_texture(texture: Texture2D) {
    unsafe {
        UnloadTexture(texture)  
    }
}