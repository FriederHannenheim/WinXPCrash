use nih_plug::prelude::*;
use std::sync::Arc;

use crate::buffer::RingBuffer;

mod buffer;

const MAX_BUFFER_SIZE: usize = 65536;


pub struct WinXpCrash {
    params: Arc<WinXpCrashParams>,

    channel_buffers: Vec<RingBuffer>,

    note_freezing: bool,
}

#[derive(Params)]
struct WinXpCrashParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "buffer_size"]
    pub buffer_size: IntParam,

    #[id = "freeze"]
    pub freeze: BoolParam,
}

impl Default for WinXpCrash {
    fn default() -> Self {
        Self {
            params: Arc::new(WinXpCrashParams::default()),
            channel_buffers: vec![],
            note_freezing: false,
        }
    }
}

impl Default for WinXpCrashParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            buffer_size: IntParam::new(
                "Buffer Size",
                1024,
                IntRange::Linear { min: 128, max: MAX_BUFFER_SIZE as i32 }
            ),
            freeze: BoolParam::new(
                "Freeze",
                false,
            )
        }
    }
}

impl Plugin for WinXpCrash {
    const NAME: &'static str = "Windows XP Crash";
    const VENDOR: &'static str = "Frieder Hannenheim";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "frieder12.fml@pm.me";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    },
    AudioIOLayout {
        main_input_channels: NonZeroU32::new(1),
        main_output_channels: NonZeroU32::new(1),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    },
    ];


    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let default_buffer = RingBuffer::new(self.params.buffer_size.value() as usize);
        let num_channels = Into::<u32>::into(audio_io_layout.main_input_channels.unwrap());

        self.channel_buffers = vec![default_buffer; num_channels as usize];

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn { note, .. } => {
                    self.note_freezing = true;
                },
                NoteEvent::NoteOff { note, .. } => {
                    self.note_freezing = false;
                },
                _ => {},
            }
        }

        for mut channel_sample in buffer.iter_samples() {
            for (i, channel_buffer) in self.channel_buffers.iter_mut().enumerate() {
                let sample = channel_sample.get_mut(i).expect("More buffers than channels created");
                *sample = channel_buffer.next_item(*sample);
            }
        }

        for channel_buffer in self.channel_buffers.iter_mut() {
            channel_buffer.resize(self.params.buffer_size.value() as usize);
            channel_buffer.freezing = self.params.freeze.value() || self.note_freezing;
        } 

        ProcessStatus::Normal
    }
}

impl ClapPlugin for WinXpCrash {
    const CLAP_ID: &'static str = "net.fhannenheim.win-xp-crash";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("The sound of a Windows XP PC crashing as audioplugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Glitch,
    ];
}

impl Vst3Plugin for WinXpCrash {
    const VST3_CLASS_ID: [u8; 16] = *b"WinXpCrash123456";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(WinXpCrash);
nih_export_vst3!(WinXpCrash);
