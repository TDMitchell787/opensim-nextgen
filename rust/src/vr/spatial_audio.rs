// OpenSim Next - Phase 32.3 Spatial Audio & 3D Sound Processing
// Revolutionary spatial audio engine with 3D positional audio, real-time reverb, and acoustic modeling
// Supporting binaural audio, room acoustics simulation, and multi-channel audio systems

use crate::vr::{VRError, Pose3D, SpatialAudioUpdate, SpatialAudioSource, RoomAcoustics};
use crate::monitoring::metrics::MetricsCollector;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SpatialAudioEngine {
    config: SpatialAudioConfig,
    audio_sessions: Arc<RwLock<HashMap<Uuid, AudioSession>>>,
    audio_mixer: Arc<AudioMixer>,
    spatial_processor: Arc<SpatialProcessor>,
    reverb_engine: Arc<ReverbEngine>,
    hrtf_processor: Arc<HRTFProcessor>,
    room_acoustics: Arc<RoomAcoustics3D>,
    audio_streaming: Arc<AudioStreaming>,
    metrics: Arc<MetricsCollector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAudioConfig {
    pub enabled: bool,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub channels: u32,
    pub bit_depth: u32,
    pub latency_target_ms: f32,
    pub hrtf_enabled: bool,
    pub reverb_enabled: bool,
    pub doppler_effect_enabled: bool,
    pub distance_attenuation_enabled: bool,
    pub occlusion_enabled: bool,
    pub air_absorption_enabled: bool,
    pub max_audio_sources: u32,
    pub spatial_resolution: f32,
    pub processing_quality: AudioQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub listener_pose: Pose3D,
    pub audio_sources: HashMap<String, AudioSource>,
    pub room_model: RoomModel,
    pub user_preferences: AudioPreferences,
    pub performance_stats: AudioPerformanceStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSource {
    pub source_id: String,
    pub source_type: AudioSourceType,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub volume: f32,
    pub pitch: f32,
    pub distance_attenuation: DistanceAttenuation,
    pub directivity_pattern: DirectivityPattern,
    pub audio_data: AudioData,
    pub looping: bool,
    pub spatial_blend: f32, // 0.0 = 2D, 1.0 = 3D
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioSourceType {
    Voice,
    Music,
    SoundEffect,
    Environment,
    UI,
    Notification,
    Footsteps,
    Impact,
    Ambient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceAttenuation {
    pub attenuation_model: AttenuationModel,
    pub reference_distance: f32,
    pub max_distance: f32,
    pub rolloff_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttenuationModel {
    None,
    Linear,
    Inverse,
    InverseSquare,
    Logarithmic,
    Custom(Vec<f32>), // Custom curve points
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectivityPattern {
    pub pattern_type: DirectivityType,
    pub cone_angle: f32,
    pub cone_outer_angle: f32,
    pub cone_outer_gain: f32,
    pub orientation: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectivityType {
    Omnidirectional,
    Directional,
    Bidirectional,
    Cardioid,
    Custom(Vec<DirectivityPoint>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectivityPoint {
    pub angle: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    pub format: AudioFormat,
    pub duration_ms: u32,
    pub channels: u32,
    pub sample_rate: u32,
    pub bit_rate: u32,
    pub compression: AudioCompression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    WAV,
    MP3,
    OGG,
    FLAC,
    PCM,
    AAC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioCompression {
    None,
    Lossless,
    Lossy(f32), // Quality factor
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomModel {
    pub room_dimensions: [f32; 3], // Length, Width, Height
    pub wall_materials: [Material; 6], // 6 walls (including floor and ceiling)
    pub room_shape: RoomShape,
    pub furniture: Vec<FurnitureItem>,
    pub environmental_factors: EnvironmentalFactors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub material_type: MaterialType,
    pub absorption_coefficients: Vec<FrequencyResponse>, // Per frequency band
    pub reflection_coefficient: f32,
    pub scattering_coefficient: f32,
    pub transmission_coefficient: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterialType {
    Concrete,
    Wood,
    Carpet,
    Glass,
    Metal,
    Fabric,
    Acoustic,
    Air,
    Water,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyResponse {
    pub frequency_hz: f32,
    pub response: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomShape {
    Rectangular,
    Circular,
    LShaped,
    Complex(Vec<[f32; 3]>), // Vertex list for complex shapes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FurnitureItem {
    pub item_type: FurnitureType,
    pub position: [f32; 3],
    pub dimensions: [f32; 3],
    pub material: Material,
    pub acoustic_properties: AcousticProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FurnitureType {
    Chair,
    Table,
    Sofa,
    Bookshelf,
    Curtains,
    Painting,
    TV,
    Speaker,
    Plant,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcousticProperties {
    pub sound_absorption: f32,
    pub sound_scattering: f32,
    pub resonant_frequencies: Vec<f32>,
    pub acoustic_shadow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalFactors {
    pub temperature: f32,
    pub humidity: f32,
    pub air_pressure: f32,
    pub air_density: f32,
    pub wind_velocity: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPreferences {
    pub master_volume: f32,
    pub voice_volume: f32,
    pub music_volume: f32,
    pub effects_volume: f32,
    pub hearing_profile: HearingProfile,
    pub accessibility_settings: AudioAccessibilitySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HearingProfile {
    pub hearing_loss: Vec<FrequencyHearingLoss>,
    pub tinnitus_frequency: Option<f32>,
    pub dynamic_range_compression: bool,
    pub frequency_compensation: bool,
    pub preferred_frequency_response: Vec<FrequencyResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyHearingLoss {
    pub frequency_hz: f32,
    pub loss_db: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAccessibilitySettings {
    pub visual_audio_indicators: bool,
    pub haptic_audio_substitution: bool,
    pub audio_description_enabled: bool,
    pub captions_enabled: bool,
    pub sign_language_avatar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioPerformanceStats {
    pub latency_ms: f32,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: f32,
    pub dropped_samples: u64,
    pub buffer_underruns: u64,
    pub processing_time_ms: f32,
}

#[derive(Debug)]
pub struct AudioMixer {
    config: MixerConfig,
    input_channels: Vec<InputChannel>,
    output_channels: Vec<OutputChannel>,
    effects_chain: Vec<AudioEffect>,
    mixer_matrix: MixerMatrix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerConfig {
    pub max_input_channels: u32,
    pub max_output_channels: u32,
    pub internal_sample_rate: u32,
    pub internal_bit_depth: u32,
    pub mixer_latency_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputChannel {
    pub channel_id: String,
    pub channel_type: ChannelType,
    pub gain: f32,
    pub mute: bool,
    pub solo: bool,
    pub effects: Vec<AudioEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    Mono,
    Stereo,
    Surround5_1,
    Surround7_1,
    Ambisonics,
    Binaural,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputChannel {
    pub channel_id: String,
    pub channel_type: ChannelType,
    pub routing: OutputRouting,
    pub master_gain: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputRouting {
    Headphones,
    Speakers,
    HapticDevice,
    NetworkStream,
    FileOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEffect {
    pub effect_type: EffectType,
    pub parameters: HashMap<String, f32>,
    pub enabled: bool,
    pub bypass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectType {
    Equalizer,
    Compressor,
    Limiter,
    Reverb,
    Delay,
    Chorus,
    Flanger,
    Phaser,
    Distortion,
    Filter,
    Spatializer,
    HRTF,
    Doppler,
    Occlusion,
}

#[derive(Debug)]
pub struct MixerMatrix {
    pub routing_matrix: Vec<Vec<f32>>, // Input to Output routing gains
    pub aux_sends: Vec<AuxSend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuxSend {
    pub send_id: String,
    pub destination: String,
    pub send_level: f32,
    pub pre_post_fader: bool,
}

#[derive(Debug)]
pub struct SpatialProcessor {
    config: SpatialConfig,
    panning_algorithms: Vec<PanningAlgorithm>,
    distance_processors: Vec<DistanceProcessor>,
    occlusion_processor: OcclusionProcessor,
    doppler_processor: DopplerProcessor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialConfig {
    pub panning_algorithm: PanningMethod,
    pub speaker_configuration: SpeakerConfiguration,
    pub head_tracking_enabled: bool,
    pub crossfeed_enabled: bool,
    pub crossfeed_amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanningMethod {
    StereoPanning,
    VectorBasedAmplitudePanning,
    Ambisonics,
    WaveFieldSynthesis,
    Binaural,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerConfiguration {
    pub speakers: Vec<Speaker>,
    pub configuration_type: ConfigurationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speaker {
    pub speaker_id: String,
    pub position: [f32; 3],
    pub orientation: [f32; 3],
    pub frequency_response: Vec<FrequencyResponse>,
    pub max_output_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigurationType {
    Stereo,
    Quadraphonic,
    Surround5_1,
    Surround7_1,
    Surround9_1,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanningAlgorithm {
    pub algorithm_name: String,
    pub algorithm_type: PanningMethod,
    pub parameters: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceProcessor {
    pub processor_name: String,
    pub attenuation_model: AttenuationModel,
    pub air_absorption_model: AirAbsorptionModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirAbsorptionModel {
    pub enabled: bool,
    pub absorption_coefficients: Vec<FrequencyAbsorption>,
    pub temperature_compensation: bool,
    pub humidity_compensation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyAbsorption {
    pub frequency_hz: f32,
    pub absorption_db_per_meter: f32,
}

#[derive(Debug)]
pub struct OcclusionProcessor {
    occlusion_models: Vec<OcclusionModel>,
    raytracing_engine: RaytracingEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionModel {
    pub model_name: String,
    pub occlusion_type: OcclusionType,
    pub filter_parameters: FilterParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OcclusionType {
    SimpleObstruction,
    FrequencyDependentObstruction,
    GeometricObstruction,
    RaytracingObstruction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterParameters {
    pub filter_type: FilterType,
    pub cutoff_frequency: f32,
    pub resonance: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    BandStop,
    Notch,
    AllPass,
}

#[derive(Debug)]
pub struct RaytracingEngine {
    ray_count: u32,
    max_reflections: u32,
    reflection_accuracy: f32,
    diffraction_enabled: bool,
}

#[derive(Debug)]
pub struct DopplerProcessor {
    config: DopplerConfig,
    velocity_history: HashMap<String, Vec<VelocityPoint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DopplerConfig {
    pub enabled: bool,
    pub speed_of_sound: f32,
    pub max_doppler_factor: f32,
    pub smoothing_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityPoint {
    pub timestamp: f64,
    pub velocity: [f32; 3],
}

#[derive(Debug)]
pub struct ReverbEngine {
    reverb_algorithms: Vec<ReverbAlgorithm>,
    impulse_responses: HashMap<String, ImpulseResponse>,
    convolution_engine: ConvolutionEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverbAlgorithm {
    pub algorithm_name: String,
    pub algorithm_type: ReverbType,
    pub parameters: ReverbParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReverbType {
    Algorithmic,
    Convolution,
    PlateReverb,
    SpringReverb,
    HallReverb,
    RoomReverb,
    ChamberReverb,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverbParameters {
    pub room_size: f32,
    pub decay_time: f32,
    pub early_reflections_level: f32,
    pub late_reflections_level: f32,
    pub diffusion: f32,
    pub density: f32,
    pub high_frequency_damping: f32,
    pub low_frequency_damping: f32,
    pub pre_delay: f32,
}

#[derive(Debug)]
pub struct ImpulseResponse {
    pub name: String,
    pub sample_rate: u32,
    pub length_samples: u32,
    pub channels: u32,
    pub data: Vec<f32>,
    pub metadata: ImpulseResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpulseResponseMetadata {
    pub recording_location: String,
    pub microphone_position: [f32; 3],
    pub source_position: [f32; 3],
    pub room_dimensions: [f32; 3],
    pub recording_conditions: RecordingConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConditions {
    pub temperature: f32,
    pub humidity: f32,
    pub background_noise_level: f32,
    pub recording_equipment: String,
}

#[derive(Debug)]
pub struct ConvolutionEngine {
    fft_size: u32,
    overlap_add_buffers: Vec<Vec<f32>>,
    fft_buffers: Vec<Vec<f32>>,
}

#[derive(Debug)]
pub struct HRTFProcessor {
    hrtf_database: HRTFDatabase,
    interpolation_engine: InterpolationEngine,
    head_tracking: HeadTracking,
}

#[derive(Debug)]
pub struct HRTFDatabase {
    subject_databases: HashMap<String, SubjectHRTF>,
    default_subject: String,
    anthropometric_scaling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectHRTF {
    pub subject_id: String,
    pub anthropometric_data: AnthropometricData,
    pub hrtf_measurements: HashMap<String, HRTFMeasurement>,
    pub measurement_grid: MeasurementGrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropometricData {
    pub head_circumference: f32,
    pub head_width: f32,
    pub head_depth: f32,
    pub pinna_height: f32,
    pub pinna_width: f32,
    pub cavum_concha_height: f32,
    pub cavum_concha_width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HRTFMeasurement {
    pub azimuth: f32,
    pub elevation: f32,
    pub distance: f32,
    pub left_ear_ir: Vec<f32>,
    pub right_ear_ir: Vec<f32>,
    pub measurement_quality: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementGrid {
    pub azimuth_resolution: f32,
    pub elevation_resolution: f32,
    pub distance_points: Vec<f32>,
    pub total_measurements: u32,
}

#[derive(Debug)]
pub struct InterpolationEngine {
    interpolation_method: InterpolationMethod,
    cache_size: u32,
    interpolation_cache: HashMap<String, InterpolatedHRTF>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterpolationMethod {
    NearestNeighbor,
    LinearInterpolation,
    CubicInterpolation,
    SphericalInterpolation,
    BarycentricInterpolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpolatedHRTF {
    pub position_key: String,
    pub left_ear_ir: Vec<f32>,
    pub right_ear_ir: Vec<f32>,
    pub interpolation_weights: Vec<f32>,
    pub cache_timestamp: u64,
}

#[derive(Debug)]
pub struct HeadTracking {
    pub tracking_enabled: bool,
    pub prediction_enabled: bool,
    pub prediction_time_ms: f32,
    pub smoothing_factor: f32,
    pub head_pose_history: Vec<HeadPosePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadPosePoint {
    pub timestamp: f64,
    pub pose: Pose3D,
}

#[derive(Debug)]
pub struct RoomAcoustics3D {
    acoustic_models: Vec<AcousticModel>,
    raytracing_processor: RaytracingProcessor,
    image_source_processor: ImageSourceProcessor,
    finite_element_processor: Option<FiniteElementProcessor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcousticModel {
    pub model_name: String,
    pub model_type: AcousticModelType,
    pub accuracy: ModelAccuracy,
    pub computational_cost: ComputationalCost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcousticModelType {
    GeometricAcoustics,
    WaveAcoustics,
    HybridModel,
    StatisticalModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelAccuracy {
    Low,
    Medium,
    High,
    Research,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputationalCost {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug)]
pub struct RaytracingProcessor {
    ray_count: u32,
    max_reflections: u32,
    diffraction_order: u32,
    scattering_enabled: bool,
    atmospheric_attenuation: bool,
}

#[derive(Debug)]
pub struct ImageSourceProcessor {
    max_image_sources: u32,
    visibility_threshold: f32,
    frequency_dependent_reflections: bool,
}

#[derive(Debug)]
pub struct FiniteElementProcessor {
    mesh_resolution: f32,
    frequency_range: (f32, f32),
    solver_type: SolverType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolverType {
    DirectSolver,
    IterativeSolver,
    MultigridSolver,
}

#[derive(Debug)]
pub struct AudioStreaming {
    streaming_config: StreamingConfig,
    network_buffers: NetworkBuffers,
    compression_engine: CompressionEngine,
    synchronization: StreamingSynchronization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub enabled: bool,
    pub target_bitrate: u32,
    pub adaptive_bitrate: bool,
    pub max_latency_ms: f32,
    pub buffer_size_ms: f32,
    pub compression_algorithm: CompressionAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    None,
    OPUS,
    AAC,
    MP3,
    FLAC,
    Custom(String),
}

#[derive(Debug)]
pub struct NetworkBuffers {
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
    jitter_buffer: JitterBuffer,
}

#[derive(Debug)]
pub struct JitterBuffer {
    buffer_size: u32,
    target_delay_ms: f32,
    adaptive_sizing: bool,
    packet_loss_concealment: bool,
}

#[derive(Debug)]
pub struct CompressionEngine {
    encoders: HashMap<CompressionAlgorithm, Box<dyn AudioEncoder>>,
    decoders: HashMap<CompressionAlgorithm, Box<dyn AudioDecoder>>,
}

pub trait AudioEncoder: std::fmt::Debug {
    fn encode(&self, input: &[f32]) -> Result<Vec<u8>, AudioError>;
    fn get_latency(&self) -> f32;
}

pub trait AudioDecoder: std::fmt::Debug {
    fn decode(&self, input: &[u8]) -> Result<Vec<f32>, AudioError>;
    fn get_latency(&self) -> f32;
}

#[derive(Debug)]
pub struct StreamingSynchronization {
    clock_sync: ClockSynchronization,
    buffer_management: BufferManagement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockSynchronization {
    pub master_clock: bool,
    pub sync_interval_ms: u32,
    pub clock_drift_compensation: bool,
    pub network_jitter_compensation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferManagement {
    pub underrun_recovery: UnderrunRecovery,
    pub overrun_recovery: OverrunRecovery,
    pub adaptive_buffering: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnderrunRecovery {
    SilenceInsertion,
    BufferStretching,
    SampleRateConversion,
    PredictiveRecovery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverrunRecovery {
    SampleDropping,
    BufferCompression,
    QualityReduction,
    AdaptiveCompression,
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Audio device not available: {0}")]
    DeviceNotAvailable(String),
    #[error("Sample rate not supported: {0}")]
    UnsupportedSampleRate(u32),
    #[error("Buffer underrun")]
    BufferUnderrun,
    #[error("Buffer overrun")]
    BufferOverrun,
    #[error("Processing error: {0}")]
    ProcessingError(String),
    #[error("HRTF not found for position: azimuth={0}, elevation={1}")]
    HRTFNotFound(f32, f32),
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

impl SpatialAudioEngine {
    pub async fn new(
        config: crate::vr::VRConfig,
        metrics: Arc<MetricsCollector>,
    ) -> Result<Arc<Self>, VRError> {
        let spatial_config = SpatialAudioConfig {
            enabled: config.spatial_audio_enabled,
            sample_rate: config.audio_sample_rate,
            buffer_size: 512, // Reasonable default
            channels: 2, // Stereo by default
            bit_depth: 32, // 32-bit float
            latency_target_ms: 10.0, // Low latency for VR
            hrtf_enabled: true,
            reverb_enabled: true,
            doppler_effect_enabled: true,
            distance_attenuation_enabled: true,
            occlusion_enabled: true,
            air_absorption_enabled: true,
            max_audio_sources: 128,
            spatial_resolution: 1.0, // 1 degree resolution
            processing_quality: AudioQuality::High,
        };

        let audio_mixer = Arc::new(AudioMixer {
            config: MixerConfig {
                max_input_channels: 64,
                max_output_channels: 8,
                internal_sample_rate: spatial_config.sample_rate,
                internal_bit_depth: spatial_config.bit_depth,
                mixer_latency_ms: 2.0,
            },
            input_channels: Vec::new(),
            output_channels: Vec::new(),
            effects_chain: Vec::new(),
            mixer_matrix: MixerMatrix {
                routing_matrix: Vec::new(),
                aux_sends: Vec::new(),
            },
        });

        let spatial_processor = Arc::new(SpatialProcessor {
            config: SpatialConfig {
                panning_algorithm: PanningMethod::Binaural,
                speaker_configuration: Self::create_default_speaker_config(),
                head_tracking_enabled: true,
                crossfeed_enabled: false,
                crossfeed_amount: 0.3,
            },
            panning_algorithms: Self::create_panning_algorithms(),
            distance_processors: Self::create_distance_processors(),
            occlusion_processor: OcclusionProcessor {
                occlusion_models: Self::create_occlusion_models(),
                raytracing_engine: RaytracingEngine {
                    ray_count: 512,
                    max_reflections: 3,
                    reflection_accuracy: 0.1,
                    diffraction_enabled: true,
                },
            },
            doppler_processor: DopplerProcessor {
                config: DopplerConfig {
                    enabled: spatial_config.doppler_effect_enabled,
                    speed_of_sound: 343.0, // m/s at 20°C
                    max_doppler_factor: 2.0,
                    smoothing_factor: 0.9,
                },
                velocity_history: HashMap::new(),
            },
        });

        let reverb_engine = Arc::new(ReverbEngine {
            reverb_algorithms: Self::create_reverb_algorithms(),
            impulse_responses: Self::load_impulse_responses(),
            convolution_engine: ConvolutionEngine {
                fft_size: 2048,
                overlap_add_buffers: Vec::new(),
                fft_buffers: Vec::new(),
            },
        });

        let hrtf_processor = Arc::new(HRTFProcessor {
            hrtf_database: Self::create_hrtf_database(),
            interpolation_engine: InterpolationEngine {
                interpolation_method: InterpolationMethod::BarycentricInterpolation,
                cache_size: 1024,
                interpolation_cache: HashMap::new(),
            },
            head_tracking: HeadTracking {
                tracking_enabled: true,
                prediction_enabled: true,
                prediction_time_ms: 20.0,
                smoothing_factor: 0.8,
                head_pose_history: Vec::new(),
            },
        });

        let room_acoustics = Arc::new(RoomAcoustics3D {
            acoustic_models: Self::create_acoustic_models(),
            raytracing_processor: RaytracingProcessor {
                ray_count: 1024,
                max_reflections: 5,
                diffraction_order: 1,
                scattering_enabled: true,
                atmospheric_attenuation: true,
            },
            image_source_processor: ImageSourceProcessor {
                max_image_sources: 32,
                visibility_threshold: 0.01,
                frequency_dependent_reflections: true,
            },
            finite_element_processor: None, // Too computationally expensive for real-time
        });

        let audio_streaming = Arc::new(AudioStreaming {
            streaming_config: StreamingConfig {
                enabled: false, // Disabled by default
                target_bitrate: 320000, // 320 kbps
                adaptive_bitrate: true,
                max_latency_ms: 50.0,
                buffer_size_ms: 100.0,
                compression_algorithm: CompressionAlgorithm::OPUS,
            },
            network_buffers: NetworkBuffers {
                input_buffer: Vec::new(),
                output_buffer: Vec::new(),
                jitter_buffer: JitterBuffer {
                    buffer_size: 1024,
                    target_delay_ms: 50.0,
                    adaptive_sizing: true,
                    packet_loss_concealment: true,
                },
            },
            compression_engine: CompressionEngine {
                encoders: HashMap::new(),
                decoders: HashMap::new(),
            },
            synchronization: StreamingSynchronization {
                clock_sync: ClockSynchronization {
                    master_clock: false,
                    sync_interval_ms: 1000,
                    clock_drift_compensation: true,
                    network_jitter_compensation: true,
                },
                buffer_management: BufferManagement {
                    underrun_recovery: UnderrunRecovery::PredictiveRecovery,
                    overrun_recovery: OverrunRecovery::AdaptiveCompression,
                    adaptive_buffering: true,
                },
            },
        });

        let engine = Self {
            config: spatial_config,
            audio_sessions: Arc::new(RwLock::new(HashMap::new())),
            audio_mixer,
            spatial_processor,
            reverb_engine,
            hrtf_processor,
            room_acoustics,
            audio_streaming,
            metrics,
        };

        Ok(Arc::new(engine))
    }

    pub async fn create_audio_session(&self, session_id: Uuid, user_id: Uuid) -> Result<(), VRError> {
        let session = AudioSession {
            session_id,
            user_id,
            listener_pose: Pose3D {
                position: [0.0, 0.0, 0.0],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            audio_sources: HashMap::new(),
            room_model: Self::create_default_room_model(),
            user_preferences: AudioPreferences {
                master_volume: 1.0,
                voice_volume: 1.0,
                music_volume: 0.8,
                effects_volume: 0.9,
                hearing_profile: HearingProfile {
                    hearing_loss: Vec::new(),
                    tinnitus_frequency: None,
                    dynamic_range_compression: false,
                    frequency_compensation: false,
                    preferred_frequency_response: Vec::new(),
                },
                accessibility_settings: AudioAccessibilitySettings {
                    visual_audio_indicators: false,
                    haptic_audio_substitution: false,
                    audio_description_enabled: false,
                    captions_enabled: false,
                    sign_language_avatar: false,
                },
            },
            performance_stats: AudioPerformanceStats::default(),
        };

        {
            let mut sessions = self.audio_sessions.write().await;
            sessions.insert(session_id, session);
        }

        self.metrics.record_spatial_audio_session_created(session_id).await;
        Ok(())
    }

    pub async fn destroy_audio_session(&self, session_id: Uuid) -> Result<(), VRError> {
        {
            let mut sessions = self.audio_sessions.write().await;
            sessions.remove(&session_id);
        }

        self.metrics.record_spatial_audio_session_destroyed(session_id).await;
        Ok(())
    }

    pub async fn process_spatial_frame(&self, session_id: Uuid, frame_data: &crate::vr::VRFrameData) -> Result<SpatialAudioUpdate, VRError> {
        let session = {
            let sessions = self.audio_sessions.read().await;
            sessions.get(&session_id).cloned()
        };

        let session = session.ok_or(VRError::SessionNotFound(session_id))?;

        // Update listener position from VR frame data
        let listener_pose = Pose3D {
            position: frame_data.head_pose.position,
            orientation: frame_data.head_pose.orientation,
        };

        // Process all audio sources with spatial audio
        let mut audio_sources = Vec::new();
        for (source_id, source) in &session.audio_sources {
            let processed_source = self.process_audio_source(source, &listener_pose, &session.room_model).await?;
            audio_sources.push(processed_source);
        }

        // Calculate room acoustics
        let room_acoustics = self.calculate_room_acoustics(&session.room_model, &listener_pose).await?;

        let update = SpatialAudioUpdate {
            audio_sources,
            listener_position: listener_pose,
            room_acoustics,
        };

        self.metrics.record_spatial_audio_frame_processed(session_id).await;
        Ok(update)
    }

    async fn process_audio_source(&self, source: &AudioSource, listener_pose: &Pose3D, room_model: &RoomModel) -> Result<SpatialAudioSource, VRError> {
        // Calculate distance and direction
        let source_to_listener = [
            listener_pose.position[0] - source.position[0],
            listener_pose.position[1] - source.position[1],
            listener_pose.position[2] - source.position[2],
        ];
        
        let distance = (source_to_listener[0].powi(2) + source_to_listener[1].powi(2) + source_to_listener[2].powi(2)).sqrt();

        // Apply distance attenuation
        let distance_attenuation = self.calculate_distance_attenuation(distance, &source.distance_attenuation);

        // Calculate frequency response (simplified)
        let frequency_response = vec![1.0; 10]; // 10 frequency bands

        Ok(SpatialAudioSource {
            source_id: source.source_id.clone(),
            position: source.position,
            volume: source.volume * distance_attenuation,
            frequency_response,
            distance_attenuation,
        })
    }

    fn calculate_distance_attenuation(&self, distance: f32, attenuation: &DistanceAttenuation) -> f32 {
        match &attenuation.attenuation_model {
            AttenuationModel::None => 1.0,
            AttenuationModel::Linear => {
                if distance <= attenuation.reference_distance {
                    1.0
                } else {
                    1.0 - ((distance - attenuation.reference_distance) / (attenuation.max_distance - attenuation.reference_distance))
                }
            }
            AttenuationModel::Inverse => {
                attenuation.reference_distance / (attenuation.reference_distance + attenuation.rolloff_factor * (distance - attenuation.reference_distance))
            }
            AttenuationModel::InverseSquare => {
                (attenuation.reference_distance / distance).powi(2)
            }
            AttenuationModel::Logarithmic => {
                if distance <= attenuation.reference_distance {
                    1.0
                } else {
                    (attenuation.reference_distance / distance).log10() * attenuation.rolloff_factor
                }
            }
            AttenuationModel::Custom(_curve) => {
                // Implement custom curve interpolation
                1.0 // Simplified
            }
        }
    }

    async fn calculate_room_acoustics(&self, room_model: &RoomModel, _listener_pose: &Pose3D) -> Result<RoomAcoustics, VRError> {
        // Simplified room acoustics calculation
        let reverb_time = self.calculate_reverb_time(room_model);
        let absorption_coefficients = self.calculate_absorption_coefficients(room_model);
        
        Ok(RoomAcoustics {
            reverb_time,
            absorption_coefficients,
            reflection_delay: 0.05, // 50ms default
        })
    }

    fn calculate_reverb_time(&self, room_model: &RoomModel) -> f32 {
        // Sabine formula: RT60 = 0.161 * V / A
        let volume = room_model.room_dimensions[0] * room_model.room_dimensions[1] * room_model.room_dimensions[2];
        let surface_area = 2.0 * (room_model.room_dimensions[0] * room_model.room_dimensions[1] + 
                                  room_model.room_dimensions[1] * room_model.room_dimensions[2] + 
                                  room_model.room_dimensions[2] * room_model.room_dimensions[0]);
        
        // Average absorption coefficient (simplified)
        let avg_absorption = room_model.wall_materials.iter()
            .map(|m| m.absorption_coefficients.get(0).map_or(0.1, |f| f.response))
            .sum::<f32>() / room_model.wall_materials.len() as f32;
        
        let total_absorption = surface_area * avg_absorption;
        
        if total_absorption > 0.0 {
            0.161 * volume / total_absorption
        } else {
            2.0 // Default reverb time
        }
    }

    fn calculate_absorption_coefficients(&self, room_model: &RoomModel) -> Vec<f32> {
        // Return absorption coefficients for different frequency bands
        vec![0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45] // 8 frequency bands
    }

    pub fn is_healthy(&self) -> bool {
        self.config.enabled
    }

    // Helper methods for initialization
    fn create_default_speaker_config() -> SpeakerConfiguration {
        SpeakerConfiguration {
            speakers: vec![
                Speaker {
                    speaker_id: "left".to_string(),
                    position: [-1.0, 0.0, 0.0],
                    orientation: [1.0, 0.0, 0.0],
                    frequency_response: vec![
                        FrequencyResponse { frequency_hz: 20.0, response: 1.0 },
                        FrequencyResponse { frequency_hz: 20000.0, response: 1.0 },
                    ],
                    max_output_level: 120.0, // dB SPL
                },
                Speaker {
                    speaker_id: "right".to_string(),
                    position: [1.0, 0.0, 0.0],
                    orientation: [-1.0, 0.0, 0.0],
                    frequency_response: vec![
                        FrequencyResponse { frequency_hz: 20.0, response: 1.0 },
                        FrequencyResponse { frequency_hz: 20000.0, response: 1.0 },
                    ],
                    max_output_level: 120.0, // dB SPL
                },
            ],
            configuration_type: ConfigurationType::Stereo,
        }
    }

    fn create_panning_algorithms() -> Vec<PanningAlgorithm> {
        vec![
            PanningAlgorithm {
                algorithm_name: "HRTF Binaural".to_string(),
                algorithm_type: PanningMethod::Binaural,
                parameters: HashMap::new(),
            },
        ]
    }

    fn create_distance_processors() -> Vec<DistanceProcessor> {
        vec![
            DistanceProcessor {
                processor_name: "Inverse Square Law".to_string(),
                attenuation_model: AttenuationModel::InverseSquare,
                air_absorption_model: AirAbsorptionModel {
                    enabled: true,
                    absorption_coefficients: vec![
                        FrequencyAbsorption { frequency_hz: 1000.0, absorption_db_per_meter: 0.1 },
                        FrequencyAbsorption { frequency_hz: 4000.0, absorption_db_per_meter: 0.3 },
                        FrequencyAbsorption { frequency_hz: 8000.0, absorption_db_per_meter: 0.8 },
                    ],
                    temperature_compensation: true,
                    humidity_compensation: true,
                },
            },
        ]
    }

    fn create_occlusion_models() -> Vec<OcclusionModel> {
        vec![
            OcclusionModel {
                model_name: "Low Pass Filter".to_string(),
                occlusion_type: OcclusionType::FrequencyDependentObstruction,
                filter_parameters: FilterParameters {
                    filter_type: FilterType::LowPass,
                    cutoff_frequency: 1000.0,
                    resonance: 0.7,
                    gain: 0.5,
                },
            },
        ]
    }

    fn create_reverb_algorithms() -> Vec<ReverbAlgorithm> {
        vec![
            ReverbAlgorithm {
                algorithm_name: "Room Reverb".to_string(),
                algorithm_type: ReverbType::RoomReverb,
                parameters: ReverbParameters {
                    room_size: 0.5,
                    decay_time: 1.5,
                    early_reflections_level: 0.3,
                    late_reflections_level: 0.7,
                    diffusion: 0.8,
                    density: 0.9,
                    high_frequency_damping: 0.2,
                    low_frequency_damping: 0.1,
                    pre_delay: 0.03,
                },
            },
        ]
    }

    fn load_impulse_responses() -> HashMap<String, ImpulseResponse> {
        // In a real implementation, this would load actual impulse response files
        HashMap::new()
    }

    fn create_hrtf_database() -> HRTFDatabase {
        HRTFDatabase {
            subject_databases: HashMap::new(),
            default_subject: "generic".to_string(),
            anthropometric_scaling: true,
        }
    }

    fn create_acoustic_models() -> Vec<AcousticModel> {
        vec![
            AcousticModel {
                model_name: "Geometric Acoustics".to_string(),
                model_type: AcousticModelType::GeometricAcoustics,
                accuracy: ModelAccuracy::High,
                computational_cost: ComputationalCost::Medium,
            },
        ]
    }

    fn create_default_room_model() -> RoomModel {
        RoomModel {
            room_dimensions: [5.0, 4.0, 3.0], // 5m x 4m x 3m room
            wall_materials: {
                let wall_material = Material {
                    material_type: MaterialType::Concrete,
                    absorption_coefficients: vec![
                        FrequencyResponse { frequency_hz: 125.0, response: 0.01 },
                        FrequencyResponse { frequency_hz: 250.0, response: 0.01 },
                        FrequencyResponse { frequency_hz: 500.0, response: 0.02 },
                        FrequencyResponse { frequency_hz: 1000.0, response: 0.02 },
                        FrequencyResponse { frequency_hz: 2000.0, response: 0.02 },
                        FrequencyResponse { frequency_hz: 4000.0, response: 0.03 },
                    ],
                    reflection_coefficient: 0.98,
                    scattering_coefficient: 0.1,
                    transmission_coefficient: 0.01,
                };
                [
                    wall_material.clone(),
                    wall_material.clone(),
                    wall_material.clone(),
                    wall_material.clone(),
                    wall_material.clone(),
                    wall_material.clone(),
                ]
            },
            room_shape: RoomShape::Rectangular,
            furniture: Vec::new(),
            environmental_factors: EnvironmentalFactors {
                temperature: 20.0, // Celsius
                humidity: 50.0, // Percent
                air_pressure: 101325.0, // Pa
                air_density: 1.2, // kg/m³
                wind_velocity: [0.0, 0.0, 0.0],
            },
        }
    }
}

impl Default for SpatialAudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_rate: 48000,
            buffer_size: 512,
            channels: 2,
            bit_depth: 32,
            latency_target_ms: 10.0,
            hrtf_enabled: true,
            reverb_enabled: true,
            doppler_effect_enabled: true,
            distance_attenuation_enabled: true,
            occlusion_enabled: true,
            air_absorption_enabled: true,
            max_audio_sources: 128,
            spatial_resolution: 1.0,
            processing_quality: AudioQuality::High,
        }
    }
}