use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use std::io::Cursor;

pub struct ImageExporter;

impl ImageExporter {
    pub fn new() -> Self {
        Self
    }

    pub fn create_translated_image(
        &self,
        original_image_data: &[u8],
        _original_text: &str,
        translated_text: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Load the original image
        println!("Attempting to load image with {} bytes", original_image_data.len());
        let img = match image::load_from_memory(original_image_data) {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Failed to load image: {}", e);
                return Err(format!("Image loading failed: {}", e).into());
            }
        };
        let mut img = img.to_rgba8();

        // Create overlay with translated text
        self.add_text_overlay(&mut img, translated_text);

        // Convert back to bytes
        let mut buffer = Vec::new();
        {
            let mut cursor = Cursor::new(&mut buffer);
            img.write_to(&mut cursor, image::ImageFormat::Png)?;
        }

        Ok(buffer)
    }

    fn add_text_overlay(&self, img: &mut RgbaImage, text: &str) {
        let width = img.width();
        let height = img.height();
        
        // Create a text box at the bottom of the image
        let box_height = (height as f32 * 0.25) as u32; // 25% of image height
        let box_y = height - box_height;
        
        // Draw semi-transparent background with rounded corners effect
        for y in box_y..height {
            for x in 0..width {
                if let Some(pixel) = img.get_pixel_mut_checked(x, y) {
                    // Create a gradient effect
                    let alpha = if y < box_y + 10 { 
                        150 + ((y - box_y) * 10) as u8 
                    } else { 
                        200 
                    };
                    *pixel = Rgba([0, 0, 0, alpha]); // Semi-transparent black gradient
                }
            }
        }
        
        // Try to use proper font rendering, fallback to simple text
        if let Err(_) = self.draw_text_with_font(img, text, 20, box_y as i32 + 15) {
            // Fallback to simple text
            self.draw_simple_text(img, text, 20, box_y as i32 + 20);
        }
    }

    fn draw_text_with_font(&self, image: &mut RgbaImage, text: &str, x: i32, y: i32) -> Result<(), Box<dyn std::error::Error>> {
        // For now, just use the improved simple text drawing
        // In a production app, you'd load a real font file here
        self.draw_simple_text(image, text, x, y);
        Ok(())
    }

    fn draw_simple_text(&self, image: &mut RgbaImage, text: &str, x: i32, y: i32) {
        let lines: Vec<&str> = text.split('\n').take(6).collect(); // Limit to 6 lines
        
        for (line_idx, line) in lines.iter().enumerate() {
            let line_y = y + (line_idx as i32 * 22);
            let line_text = if line.len() > 65 { 
                format!("{}...", &line[..62])
            } else { 
                line.to_string()
            };
            
            self.draw_simple_text_line(image, &line_text, x, line_y);
        }
    }
    
    fn draw_simple_text_line(&self, image: &mut RgbaImage, text: &str, x: i32, y: i32) {
        // Improved simple text with better character spacing and size
        for (char_idx, ch) in text.chars().enumerate() {
            let char_x = x + (char_idx as i32 * 9); // Better spacing
            
            // Skip non-printable characters
            if !ch.is_ascii_graphic() && ch != ' ' {
                continue;
            }
            
            // Draw character as a pattern (simple but more readable)
            if ch != ' ' {
                self.draw_character_pattern(image, ch, char_x, y);
            }
        }
    }
    
    fn draw_character_pattern(&self, image: &mut RgbaImage, ch: char, x: i32, y: i32) {
        let width = image.width() as i32;
        let height = image.height() as i32;
        
        // Use a simple 5x8 bitmap font pattern for better readability
        let pattern = self.get_char_pattern(ch);
        
        for dy in 0..8 {
            for dx in 0..5 {
                let px = x + dx;
                let py = y + dy;
                
                if px >= 0 && py >= 0 && px < width && py < height {
                    let bit_index = (dy * 5 + dx) as usize;
                    if bit_index < pattern.len() && pattern[bit_index] {
                        // Draw with anti-aliasing effect
                        image.put_pixel(px as u32, py as u32, Rgba([255, 255, 255, 255]));
                        // Add slight glow effect
                        if px > 0 && py > 0 {
                            if let Some(pixel) = image.get_pixel_mut_checked((px - 1) as u32, py as u32) {
                                if pixel[3] == 0 { *pixel = Rgba([255, 255, 255, 60]); }
                            }
                            if let Some(pixel) = image.get_pixel_mut_checked(px as u32, (py - 1) as u32) {
                                if pixel[3] == 0 { *pixel = Rgba([255, 255, 255, 60]); }
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn get_char_pattern(&self, ch: char) -> Vec<bool> {
        // Simple 5x8 bitmap patterns for common characters
        match ch {
            'A' => vec![
                false, true, true, true, false,
                true, false, false, false, true,
                true, false, false, false, true,
                true, true, true, true, true,
                true, false, false, false, true,
                true, false, false, false, true,
                true, false, false, false, true,
                false, false, false, false, false,
            ],
            'B' => vec![
                true, true, true, true, false,
                true, false, false, false, true,
                true, false, false, false, true,
                true, true, true, true, false,
                true, false, false, false, true,
                true, false, false, false, true,
                true, true, true, true, false,
                false, false, false, false, false,
            ],
            'C' => vec![
                false, true, true, true, false,
                true, false, false, false, true,
                true, false, false, false, false,
                true, false, false, false, false,
                true, false, false, false, false,
                true, false, false, false, true,
                false, true, true, true, false,
                false, false, false, false, false,
            ],
            'D' => vec![
                true, true, true, false, false,
                true, false, false, true, false,
                true, false, false, false, true,
                true, false, false, false, true,
                true, false, false, false, true,
                true, false, false, true, false,
                true, true, true, false, false,
                false, false, false, false, false,
            ],
            'E' => vec![
                true, true, true, true, true,
                true, false, false, false, false,
                true, false, false, false, false,
                true, true, true, true, false,
                true, false, false, false, false,
                true, false, false, false, false,
                true, true, true, true, true,
                false, false, false, false, false,
            ],
            ' ' => vec![false; 40], // Space
            '.' => vec![
                false, false, false, false, false,
                false, false, false, false, false,
                false, false, false, false, false,
                false, false, false, false, false,
                false, false, false, false, false,
                false, false, false, false, false,
                false, true, true, false, false,
                false, true, true, false, false,
            ],
            ':' => vec![
                false, false, false, false, false,
                false, true, true, false, false,
                false, true, true, false, false,
                false, false, false, false, false,
                false, true, true, false, false,
                false, true, true, false, false,
                false, false, false, false, false,
                false, false, false, false, false,
            ],
            'I' => vec![
                true, true, true, true, true,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, true, false, false,
                true, true, true, true, true,
                false, false, false, false, false,
            ],
            'L' => vec![
                true, false, false, false, false,
                true, false, false, false, false,
                true, false, false, false, false,
                true, false, false, false, false,
                true, false, false, false, false,
                true, false, false, false, false,
                true, true, true, true, true,
                false, false, false, false, false,
            ],
            'N' => vec![
                true, false, false, false, true,
                true, true, false, false, true,
                true, false, true, false, true,
                true, false, false, true, true,
                true, false, false, false, true,
                true, false, false, false, true,
                true, false, false, false, true,
                false, false, false, false, false,
            ],
            'O' => vec![
                false, true, true, true, false,
                true, false, false, false, true,
                true, false, false, false, true,
                true, false, false, false, true,
                true, false, false, false, true,
                true, false, false, false, true,
                false, true, true, true, false,
                false, false, false, false, false,
            ],
            'R' => vec![
                true, true, true, true, false,
                true, false, false, false, true,
                true, false, false, false, true,
                true, true, true, true, false,
                true, false, true, false, false,
                true, false, false, true, false,
                true, false, false, false, true,
                false, false, false, false, false,
            ],
            'S' => vec![
                false, true, true, true, false,
                true, false, false, false, true,
                true, false, false, false, false,
                false, true, true, true, false,
                false, false, false, false, true,
                true, false, false, false, true,
                false, true, true, true, false,
                false, false, false, false, false,
            ],
            'T' => vec![
                true, true, true, true, true,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, false, false, false,
            ],
            // Lowercase letters
            'a' => vec![
                false, false, false, false, false,
                false, false, false, false, false,
                false, true, true, true, false,
                false, false, false, false, true,
                false, true, true, true, true,
                true, false, false, false, true,
                false, true, true, true, true,
                false, false, false, false, false,
            ],
            'e' => vec![
                false, false, false, false, false,
                false, false, false, false, false,
                false, true, true, true, false,
                true, false, false, false, true,
                true, true, true, true, true,
                true, false, false, false, false,
                false, true, true, true, false,
                false, false, false, false, false,
            ],
            'i' => vec![
                false, false, true, false, false,
                false, false, false, false, false,
                false, true, true, false, false,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, true, false, false,
                false, true, true, true, false,
                false, false, false, false, false,
            ],
            'o' => vec![
                false, false, false, false, false,
                false, false, false, false, false,
                false, true, true, true, false,
                true, false, false, false, true,
                true, false, false, false, true,
                true, false, false, false, true,
                false, true, true, true, false,
                false, false, false, false, false,
            ],
            's' => vec![
                false, false, false, false, false,
                false, false, false, false, false,
                false, true, true, true, false,
                true, false, false, false, false,
                false, true, true, true, false,
                false, false, false, false, true,
                true, true, true, true, false,
                false, false, false, false, false,
            ],
            't' => vec![
                false, false, true, false, false,
                false, false, true, false, false,
                true, true, true, true, true,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, true, false, false,
                false, false, false, true, true,
                false, false, false, false, false,
            ],
            // Default pattern for unknown characters
            _ => vec![
                false, true, true, true, false,
                true, false, false, false, true,
                true, false, true, false, true,
                true, false, false, false, true,
                true, false, true, false, true,
                true, false, false, false, true,
                false, true, true, true, false,
                false, false, false, false, false,
            ],
        }
    }

    pub fn create_side_by_side_comparison(
        &self,
        original_image_data: &[u8],
        original_text: &str,
        translated_text: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let img = image::load_from_memory(original_image_data)?;
        let img = img.to_rgba8();
        
        let width = img.width();
        let height = img.height();
        
        // Create a wider image for side-by-side comparison
        let mut comparison = RgbaImage::new(width * 2 + 20, height + 100);
        
        // Fill with white background
        for pixel in comparison.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }
        
        // Copy original image to left side
        for (x, y, pixel) in img.enumerate_pixels() {
            comparison.put_pixel(x + 10, y + 50, *pixel);
        }
        
        // Create translated version for right side
        let translated_img_data = self.create_translated_image(original_image_data, original_text, translated_text)?;
        let translated_img = image::load_from_memory(&translated_img_data)?.to_rgba8();
        
        for (x, y, pixel) in translated_img.enumerate_pixels() {
            comparison.put_pixel(x + width + 20, y + 50, *pixel);
        }
        
        // Add simple labels using text pixels
        self.draw_simple_text(&mut comparison, "Original", 10, 10);
        self.draw_simple_text(&mut comparison, "Translated", width as i32 + 20, 10);
        
        // Convert to bytes
        let mut buffer = Vec::new();
        {
            let mut cursor = Cursor::new(&mut buffer);
            comparison.write_to(&mut cursor, image::ImageFormat::Png)?;
        }
        
        Ok(buffer)
    }
}