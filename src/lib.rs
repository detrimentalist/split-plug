use clack_extensions::params::info::ParamInfoFlags;
use clack_extensions::params::{HostParams, implementation::*, info::ParamInfoData, ParamRescanFlags, PluginParams};
use std::ffi::CStr;
use std::string::ToString;

use clack_plugin::{plugin::descriptor::PluginDescriptor, prelude::*};

use clack_extensions::log::{HostLog, LogSeverity};
use clack_extensions::audio_ports::{AudioPortFlags, AudioPortInfoData, AudioPortInfoWriter, AudioPortType, PluginAudioPorts, PluginAudioPortsImpl, };
use clack_plugin::plugin::descriptor::features::STEREO;
use clack_plugin::plugin::descriptor::StaticPluginDescriptor;
use clack_plugin::process::audio::channels::AudioBufferType;
use clack_plugin::utils::Cookie;

use clap_sys::ext::log::CLAP_EXT_LOG;
use clap_sys::ext::params::CLAP_EXT_PARAMS;

pub struct GainPlugin<'a> {
    latest_gain_value: i32,
    _host: HostAudioThreadHandle<'a>,
}

impl<'a> Plugin<'a> for GainPlugin<'a> {
    type Shared = GainPluginShared<'a>;
    type MainThread = GainPluginMainThread<'a>;

    fn get_descriptor() -> Box<dyn PluginDescriptor> {
        use clack_plugin::plugin::descriptor::features::*;

        let bytes = concat!(env!("NAME"), "\0").as_bytes();

        Box::new(StaticPluginDescriptor {
            id: CStr::from_bytes_with_nul(b"KEIK\0").unwrap(),
            name: CStr::from_bytes_with_nul(bytes).unwrap(),
            features: Some(&[UTILITY]),
            ..Default::default()
        })
    }

    fn activate(
        host: HostAudioThreadHandle<'a>,
        _main_thread: &mut GainPluginMainThread,
        _shared: &'a GainPluginShared,
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {

        //_shared._host.request_callback();

        Ok(Self {
            latest_gain_value: 0,
            _host: host,
        })
    }

    fn process(
        &mut self,
        _process: &Process,
        mut audio: Audio,
        _events: ProcessEvents,
    ) -> Result<ProcessStatus, PluginError> {
        let io = if let Some(io) = audio.zip(0, 0) {
            io
        } else {
            return Ok(ProcessStatus::ContinueIfNotQuiet);
        };

        match io {
            AudioBufferType::F32(io) => {
                // Supports safe in_place processing
                for (input, output) in io {
                    output.set(input.get() * 2.0)
                }
            }
            AudioBufferType::F64(io) => {
                // Supports safe in_place processing
                for (input, output) in io {
                    output.set(input.get() * 2.0)
                }
            }
        }

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }

    fn declare_extensions(builder: &mut PluginExtensions<Self>, _shared: &GainPluginShared) {
        builder
            .register::<PluginParams>()
            .register::<PluginAudioPorts>();
    }
}

impl<'a> PluginParamsImpl for GainPlugin<'a> {
    fn flush(
        &mut self,
        _input_parameter_changes: &InputEvents,
        _output_parameter_changes: &mut OutputEvents,
    ) {
    }
}

impl<'a> PluginAudioPortsImpl for GainPluginMainThread<'a> {
    fn count(&self, _is_input: bool) -> u32 {
        1
    }

    fn get(&self, _is_input: bool, index: u32, writer: &mut AudioPortInfoWriter) {
        if index == 0 {
            writer.set(&AudioPortInfoData {
                id: 0,
                name: b"main",
                channel_count: 2,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::STEREO),
                in_place_pair: None,
            });
        }
    }
}

pub struct GainPluginShared<'a> {
    _host: HostHandle<'a>,
}

impl<'a> PluginShared<'a> for GainPluginShared<'a> {
    fn new(host: HostHandle<'a>) -> Result<Self, PluginError> {
        Ok(Self { _host: host })
    }
}

pub struct GainPluginMainThread<'a> {
    rusting: u32,
    #[allow(unused)]
    shared: &'a GainPluginShared<'a>,
    host: HostMainThreadHandle<'a>,
    host_log: Option<&'a HostLog>,
    host_params: Option<&'a HostParams>,
    params: Vec<String>,
}

impl<'a> PluginMainThread<'a, GainPluginShared<'a>> for GainPluginMainThread<'a> {

    fn new(
        host: HostMainThreadHandle<'a>,
        shared: &'a GainPluginShared,
    ) -> Result<Self, PluginError> {

        let mut host_log = None;
        let mut host_params = None;

        if let Some(get_extension) = host.as_raw().get_extension  {
            unsafe {
                host_log = ((get_extension)(host.as_raw(), CLAP_EXT_LOG.as_ptr()) as *const HostLog).as_ref();
                host_params = ((get_extension)(host.as_raw(), CLAP_EXT_PARAMS.as_ptr()) as *const HostParams).as_ref();
            }
        }



        let params_r = env!("PARAMS").split(',');

        let params: Vec<String> = params_r.map(|s|{
            s.to_string()
        }).collect();

        //let params = vec![env!("PARAMS").to_string()];

        Ok(Self {
            rusting: 0,
            shared,
            host: host,
            params,
            host_log,
            host_params,
        })
    }

    fn on_main_thread(&mut self) {
        if let Some(host_params) = self.host_params {
            host_params.rescan(&mut self.host, ParamRescanFlags::all());
        }
    }
}

impl<'a> PluginMainThreadParams for GainPluginMainThread<'a> {

    fn count(&self) -> u32 {
        self.params.len() as u32
    }

    fn get_info(&self, param_index: u32, info: &mut ParamInfoWriter) {

        let logstr = format!("get info {} - {}\0", param_index, self.params[param_index as usize]);

            if let Some(log) = &self.host_log {
                log.log(&self.shared._host , LogSeverity::Info, CStr::from_bytes_with_nul(logstr.as_bytes()).unwrap());
            }


        info.set(&ParamInfoData {
            id: param_index,
            name: &self.params[param_index as usize],
            module: "ultra/hello",
            default_value: 0.0,
            min_value: 0.0,
            max_value: 1000.0,
            flags: ParamInfoFlags::IS_AUTOMATABLE,
            cookie: Cookie::empty(),
        });
    }

    fn get_value(&self, param_id: u32) -> Option<f64> {
        Some(self.rusting as f64)
    }

    fn value_to_text(
        &self,
        param_id: u32,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> core::fmt::Result {
        use ::core::fmt::Write;
        write!(writer, "{}", value)
    }

    fn text_to_value(&self, _param_id: u32, _text: &str) -> Option<f64> {
        None
    }

    fn flush(&mut self, _input_events: &InputEvents, _output_events: &mut OutputEvents) {
        /*let value_events = input_events.iter().filter_map(|e| match e.as_event()? {
            Event::ParamValue(v) => Some(v),
            _ => None,
        });

        for value in value_events {
            if value.param_id() == 0 {
                self.rusting = value.value() as u32;
            }
        }*/
    }
}

#[allow(non_upper_case_globals)]
#[allow(unsafe_code)]
#[no_mangle]
pub static clap_entry: PluginEntryDescriptor = SinglePluginEntry::<GainPlugin>::DESCRIPTOR;
