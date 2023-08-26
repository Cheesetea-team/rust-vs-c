#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <raylib.h>

void calculate_next_fire_frame(uint8_t fire_buffer[400*300]) {

    uint8_t old_fire_buffer[400*300] = {};
    memcpy(old_fire_buffer, fire_buffer, (size_t)400*300);

    for(int y=1; y < (300-1); y++) {
        int p = y*400;
        for(int x=1; x < (400-1); x++) {
            int i = p+x;
            fire_buffer[i] = (uint8_t)((
               10 * old_fire_buffer[i - 1]
            +  20 * old_fire_buffer[i + 0]
            +  10 * old_fire_buffer[i + 1]
            + 160 * old_fire_buffer[i - 1 + 400]
            + 320 * old_fire_buffer[i + 0 + 400]
            + 160 * old_fire_buffer[i + 1 + 400]
            ) / 680);
        }
    }
    
}

void convert_fire_buffer_to_screen(Image screen_buffer, Color palette[256], uint8_t fire_buffer[400*300]) {
    Color *pixel = screen_buffer.data;
    for(int i = 0; i < 400 * 300; i++) {
        pixel[i] = palette[fire_buffer[i]];
    }
}

void fill_bottom_with_random_ashes(uint8_t fire_buffer[400*300]) {
    for(int i = 0; i < 400; i++) {
        fire_buffer[400*299 + i] = (uint8_t)rand() % 256;
    }
}

void draw_next_frame(Image screen_buffer, Color palette[256], uint8_t fire_buffer[400*300]) {
    fill_bottom_with_random_ashes(fire_buffer);
    calculate_next_fire_frame(fire_buffer);
    convert_fire_buffer_to_screen(screen_buffer, palette, fire_buffer);
}

void generate_palette(Color palette[256]) {
    for(int i = 0; i <= 84; i++) {
        uint8_t red = i * (0xFF / 84);
        palette[i] = (Color){.r=red,.a=0xFF};
    }

    for(int i = 85; i <= 169; i++) {
        uint8_t green = (i - 85) * (0xFF / 84);
        palette[i] = (Color){.r=0xFF,.g=green,.a=0xFF};
    }

    for(int i = 170; i <= 256; i++) {
        uint8_t blue = (i - 170) * (0xFF / 85);
        palette[i] = (Color){.r=0xFF,.g=0xFF,.b=blue,.a=0xFF};
    }
}

int main() {
    InitWindow(400, 300, "HELLO WORLD!");

    Color palette[256] = {};
    generate_palette(palette);

    // El error que se vió en directo fue no inicializar esta variable :s
    // La línea original era:
    // unsigned char fire_buffer[400*300];
    uint8_t fire_buffer[400*300] = {};

    Color screen_buffer_data[400*300] = {};

    Image screen_buffer = {
        .data = screen_buffer_data,
        .width = 400,
        .height = 300,
        .format = PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
        .mipmaps = 1
    };

    Texture2D screen_buffer_texture = LoadTextureFromImage(screen_buffer);

    while(!WindowShouldClose()) {
        BeginDrawing();

        draw_next_frame(screen_buffer, palette, fire_buffer);

        // Pasar a la GPU
        UpdateTexture(screen_buffer_texture, screen_buffer.data);
        DrawTexture(screen_buffer_texture, 0, 0, WHITE);
        //DrawPixel(200, 200, (Color){.r=255,.g=0,.b=0,.a=255});

        EndDrawing();
    }

    CloseWindow();
    return 0;
}