//! FFI bindings for whisper.cpp
//! 
//! Provides Rust bindings to the whisper.cpp library for offline
//! speech-to-text transcription.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_void};
use std::path::Path;
use tracing::{debug, error, warn};

// FFI declarations for whisper.cpp
#[link(name = "whisper")]
extern "C" {
    fn whisper_init_from_file(path_model: *const c_char) -> *mut c_void;
    fn whisper_free(ctx: *mut c_void);
    fn whisper_full_default_params(strategy: c_int) -> WhisperFullParams;
    fn whisper_full(
        ctx: *mut c_void,
        params: WhisperFullParams,
        samples: *const c_float,
        n_samples: c_int,
    ) -> c_int;
    fn whisper_full_n_segments(ctx: *mut c_void) -> c_int;
    fn whisper_full_get_segment_text(ctx: *mut c_void, i_segment: c_int) -> *const c_char;
    fn whisper_full_get_segment_t0(ctx: *mut c_void, i_segment: c_int) -> i64;
    fn whisper_full_get_segment_t1(ctx: *mut c_void, i_segment: c_int) -> i64;
    fn whisper_print_system_info() -> *const c_char;
}

// Whisper strategy constants
const WHISPER_SAMPLING_GREEDY: c_int = 0;
const WHISPER_SAMPLING_BEAM_SEARCH: c_int = 1;

// Simplified whisper parameters struct
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WhisperFullParams {
    pub strategy: c_int,
    pub n_threads: c_int,
    pub n_max_text_ctx: c_int,
    pub offset_ms: c_int,
    pub duration_ms: c_int,
    pub translate: bool,
    pub no_context: bool,
    pub single_segment: bool,
    pub print_special: bool,
    pub print_progress: bool,
    pub print_realtime: bool,
    pub print_timestamps: bool,
    pub token_timestamps: bool,
    pub thold_pt: c_float,
    pub thold_ptsum: c_float,
    pub max_len: c_int,
    pub split_on_word: bool,
    pub max_tokens: c_int,
    pub speed_up: bool,
    pub audio_ctx: c_int,
    pub prompt_tokens: *mut c_int,
    pub prompt_n_tokens: c_int,
    pub language: *const c_char,
    pub detect_language: bool,
    pub suppress_blank: bool,
    pub suppress_non_speech_tokens: bool,
    pub temperature: c_float,
    pub max_initial_ts: c_float,
    pub length_penalty: c_float,
}

/// Transcript segment with timing information
#[derive(Debug, Clone)]
pub struct TranscriptSegment {
    pub text: String,
    pub start_time_ms: i64,
    pub end_time_ms: i64,
}

/// Complete transcript result
#[derive(Debug, Clone)]
pub struct TranscriptResult {
    pub segments: Vec<TranscriptSegment>,
    pub full_text: String,
    pub language: Option<String>,
    pub processing_time_ms: u64,
}

/// Whisper context wrapper
pub struct WhisperContext {
    ctx: *mut c_void,
    model_path: String,
}

impl WhisperContext {
    /// Load whisper model from file
    pub fn from_file<P: AsRef<Path>>(model_path: P) -> Result<Self, String> {
        let path_str = model_path.as_ref().to_string_lossy();
        let c_path = CString::new(path_str.as_ref())
            .map_err(|e| format!("Invalid model path: {}", e))?;
        
        debug!("Loading whisper model from: {}", path_str);
        
        unsafe {
            let ctx = whisper_init_from_file(c_path.as_ptr());
            if ctx.is_null() {
                return Err(format!("Failed to load whisper model from: {}", path_str));
            }
            
            Ok(Self {
                ctx,
                model_path: path_str.to_string(),
            })
        }
    }
    
    /// Transcribe audio samples
    pub fn transcribe(&self, samples: &[f32], language: Option<&str>) -> Result<TranscriptResult, String> {
        let start_time = std::time::Instant::now();
        
        unsafe {
            // Get default parameters
            let mut params = whisper_full_default_params(WHISPER_SAMPLING_GREEDY);
            
            // Configure parameters
            params.n_threads = std::thread::available_parallelism()
                .map(|n| n.get() as c_int)
                .unwrap_or(4);
            params.translate = false;
            params.language = if let Some(lang) = language {
                let c_lang = CString::new(lang).unwrap();
                c_lang.as_ptr()
            } else {
                std::ptr::null()
            };
            params.detect_language = language.is_none();
            params.print_progress = false;
            params.print_timestamps = true;
            params.token_timestamps = true;
            
            // Run transcription
            let result = whisper_full(
                self.ctx,
                params,
                samples.as_ptr(),
                samples.len() as c_int,
            );
            
            if result != 0 {
                return Err(format!("Whisper transcription failed with code: {}", result));
            }
            
            // Extract segments
            let n_segments = whisper_full_n_segments(self.ctx);
            let mut segments = Vec::new();
            let mut full_text = String::new();
            
            for i in 0..n_segments {
                let text_ptr = whisper_full_get_segment_text(self.ctx, i);
                if text_ptr.is_null() {
                    continue;
                }
                
                let text = CStr::from_ptr(text_ptr).to_string_lossy().to_string();
                let start_time = whisper_full_get_segment_t0(self.ctx, i);
                let end_time = whisper_full_get_segment_t1(self.ctx, i);
                
                segments.push(TranscriptSegment {
                    text: text.clone(),
                    start_time_ms: start_time,
                    end_time_ms: end_time,
                });
                
                if !full_text.is_empty() {
                    full_text.push(' ');
                }
                full_text.push_str(&text);
            }
            
            let processing_time = start_time.elapsed().as_millis() as u64;
            
            Ok(TranscriptResult {
                segments,
                full_text,
                language: language.map(|s| s.to_string()),
                processing_time_ms: processing_time,
            })
        }
    }
    
    /// Get model path
    pub fn model_path(&self) -> &str {
        &self.model_path
    }
}

impl Drop for WhisperContext {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe {
                whisper_free(self.ctx);
            }
        }
    }
}

// Ensure WhisperContext is thread-safe
unsafe impl Send for WhisperContext {}
unsafe impl Sync for WhisperContext {}

/// Get whisper system information
pub fn get_system_info() -> String {
    unsafe {
        let info_ptr = whisper_print_system_info();
        if info_ptr.is_null() {
            return "Unable to get system info".to_string();
        }
        
        CStr::from_ptr(info_ptr).to_string_lossy().to_string()
    }
}

/// Convert audio samples from various formats to f32
pub fn convert_audio_to_f32(audio_data: &[u8], format: AudioFormat) -> Vec<f32> {
    match format {
        AudioFormat::F32 => {
            // Already f32, just reinterpret bytes
            let mut samples = Vec::with_capacity(audio_data.len() / 4);
            for chunk in audio_data.chunks_exact(4) {
                let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
                samples.push(f32::from_le_bytes(bytes));
            }
            samples
        }
        AudioFormat::I16 => {
            // Convert i16 to f32
            let mut samples = Vec::with_capacity(audio_data.len() / 2);
            for chunk in audio_data.chunks_exact(2) {
                let bytes = [chunk[0], chunk[1]];
                let sample = i16::from_le_bytes(bytes) as f32 / 32768.0;
                samples.push(sample);
            }
            samples
        }
        AudioFormat::I32 => {
            // Convert i32 to f32
            let mut samples = Vec::with_capacity(audio_data.len() / 4);
            for chunk in audio_data.chunks_exact(4) {
                let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
                let sample = i32::from_le_bytes(bytes) as f32 / 2147483648.0;
                samples.push(sample);
            }
            samples
        }
    }
}

/// Supported audio formats for conversion
#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    F32,
    I16,
    I32,
}

/// Resample audio to 16kHz (whisper's expected sample rate)
pub fn resample_to_16khz(samples: &[f32], original_rate: u32) -> Vec<f32> {
    const TARGET_RATE: u32 = 16000;
    
    if original_rate == TARGET_RATE {
        return samples.to_vec();
    }
    
    // Simple linear interpolation resampling
    let ratio = original_rate as f64 / TARGET_RATE as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);
    
    for i in 0..output_len {
        let src_index = (i as f64 * ratio) as usize;
        if src_index < samples.len() {
            output.push(samples[src_index]);
        }
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_format_conversion() {
        let i16_data = vec![0u8, 128u8, 255u8, 127u8]; // Two i16 samples
        let samples = convert_audio_to_f32(&i16_data, AudioFormat::I16);
        assert_eq!(samples.len(), 2);
    }
    
    #[test]
    fn test_resampling() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let resampled = resample_to_16khz(&samples, 32000);
        assert_eq!(resampled.len(), 4); // Half the samples
    }
}
