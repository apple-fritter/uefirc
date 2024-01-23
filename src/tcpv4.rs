// DNS4: AE3D28CC-E05B-4FA1-A011-7EB55A3F1401 BDB49030
// UDP4: 3AD9DF29-4501-478D-B1F8-7F7FE70E50F3 BDB49D38
// IP4: 41D94CD2-35B6-455A-8258-D4E51334AADD BDB496A0
// TCP4: 65530BC7-A359-410F-B010-5AADC7EC2B62 BDB4CE38
// HTTP: 7A59B29B-910B-4171-8242-A85A0DF25B5B BDB4C020

use alloc::boxed::Box;
use core::alloc::Layout;
use core::ffi::c_void;
use core::intrinsics::size_of;
use core::ptr::{copy_nonoverlapping, null};
use log::info;
use uefi::{Event, Handle, Status};

use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::media::block::BlockIoProtocol;
use uefi::proto::rng::Rng;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol};
use uefi::proto::unsafe_protocol;
use crate::ipv4::{IPv4Address, IPv4ModeData};

#[derive(Debug)]
#[repr(C)]
pub struct UnmodelledPointer(pub *mut c_void);

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4AccessPoint {
    use_default_address: bool,
    station_address: IPv4Address,
    subnet_mask: IPv4Address,
    station_port: u16,
    remote_address: IPv4Address,
    remote_port: u16,
    active_flag: bool,
}

impl TCPv4AccessPoint {
    fn new() -> Self {
        Self {
            use_default_address: true,
            // These two fields are meaningless because we set use_default_address above
            //station_address: IPv4Address::new(192, 168, 0, 3),
            //subnet_mask: IPv4Address::subnet24(),
            //station_address: IPv4Address::zero(),
            //subnet_mask: IPv4Address::zero(),
            /*
            station_address: IPv4Address::new(192, 169, 0, 3),
            subnet_mask: IPv4Address::new(255, 255, 0, 0),
            station_port: 0,
            remote_address: IPv4Address::new(192, 169, 0, 1),
            remote_port: 80,
            active_flag: true,
             */
            station_address: IPv4Address::zero(),
            subnet_mask: IPv4Address::zero(),
            //station_port: 1234,
            station_port: 0,
            //remote_address: IPv4Address::zero(),
            //remote_address: IPv4Address::new(1, 0, 169, 192),
            //remote_address: IPv4Address::new(192, 169, 0, 1),
            //remote_port: 80,
            remote_address: IPv4Address::new(93, 158, 237, 2),
            remote_port: 6665,
            active_flag: true,

        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4Option {
    receive_buffer_size: u32,
    send_buffer_size: u32,
    max_syn_back_log: u32,
    connection_timeout: u32,
    data_retries: u32,
    fin_timeout: u32,
    time_wait_timeout: u32,
    keep_alive_probes: u32,
    keep_alive_time: u32,
    keep_alive_interval: u32,
    enable_nagle: bool,
    enable_time_stamp: bool,
    enable_window_scaling: bool,
    enable_selective_ack: bool,
    enable_path_mtu_discovery: bool,
}

impl TCPv4Option {
    pub(crate) fn new() -> Self {
        Self {
            /*
            receive_buffer_size: 32 * 1024,
            send_buffer_size: 32 * 1024,
            max_syn_back_log: 128,
            connection_timeout: 20_000,
            data_retries: 10,
            fin_timeout: 60_000,
            time_wait_timeout: 120_000,
            keep_alive_probes: 9,
            keep_alive_time: 7_200_000,
            keep_alive_interval: 75_000,
            enable_nagle: true,
            enable_time_stamp: true,
            enable_window_scaling: true,
            enable_selective_ack: true,
            enable_path_mtu_discovery: true,

             */
            receive_buffer_size: 1024,
            send_buffer_size: 1024,
            max_syn_back_log: 0,
            connection_timeout: 0,
            data_retries: 0,
            fin_timeout: 0,
            time_wait_timeout: 3,
            keep_alive_probes: 0,
            keep_alive_time: 0,
            keep_alive_interval: 0,
            enable_nagle: false,
            enable_time_stamp: false,
            enable_window_scaling: false,
            enable_selective_ack: false,
            enable_path_mtu_discovery: false,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4ConfigData<'a> {
    type_of_service: u8,
    time_to_live: u8,
    access_point: TCPv4AccessPoint,
    option: Option<&'a TCPv4Option>,
}

impl<'a> TCPv4ConfigData<'a> {
    pub(crate) fn new(options: Option<&'a TCPv4Option>) -> Self {
        Self {
            // Standard values
            type_of_service: 0,
            time_to_live: 255,
            access_point: TCPv4AccessPoint::new(),
            option: options,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
#[unsafe_protocol("00720665-67EB-4a99-BAF7-D3C33A1C7CC9")]
pub struct TCPv4ServiceBindingProtocol {
    pub(crate) create_child: extern "efiapi" fn(
        this: &Self,
        out_child_handle: &mut Handle,
    ) -> Status,

    destroy_child: extern "efiapi" fn(
        this: &Self,
        child_handle: Handle,
    ) -> Status,
}


#[derive(Debug)]
#[repr(C)]
#[unsafe_protocol("65530BC7-A359-410F-B010-5AADC7EC2B62")]
pub struct TCPv4Protocol {
    pub(crate) get_mode_data: extern "efiapi" fn(
        this: &Self,
        out_connection_state: Option<&mut TCPv4ConnectionState>,
        out_config_data: Option<&mut UnmodelledPointer>,
        out_ip4_mode_data: Option<&mut IPv4ModeData>,
        out_managed_network_config_data: Option<&mut UnmodelledPointer>,
        out_simple_network_mode: Option<&mut UnmodelledPointer>,
    ) -> Status,

    pub(crate) configure: extern "efiapi" fn(
        this: &Self,
        config_data: Option<&TCPv4ConfigData>,
    ) -> Status,

    routes: extern "efiapi" fn(
        this: &Self,
        delete_route: bool,
        subnet_address: &IPv4Address,
        subnet_mask: &IPv4Address,
        gateway_address: &IPv4Address,
    ) -> Status,

    pub(crate) connect: extern "efiapi" fn(
        this: &Self,
        connection_token: &TCPv4CompletionToken,
    ) -> Status,

    accept: extern "efiapi" fn(
        this: &Self,
        listen_token: &UnmodelledPointer,
    ) -> Status,

    pub(crate) transmit: extern "efiapi" fn(
        this: &Self,
        token: &TCPv4IoToken,
    ) -> Status,

    receive: extern "efiapi" fn(
        this: &Self,
        token: &UnmodelledPointer,
    ) -> Status,

    close: extern "efiapi" fn(
        this: &Self,
        close_token: &UnmodelledPointer,
    ) -> Status,

    cancel: extern "efiapi" fn(
        this: &Self,
        completion_token: &UnmodelledPointer,
    ) -> Status,

    poll: extern "efiapi" fn(this: &Self) -> Status,
}


#[repr(C)]
pub struct TCPv4IoToken<'a> {
    pub completion_token: TCPv4CompletionToken,
    packet: TCPv4Packet<'a>,
}

impl<'a> TCPv4IoToken<'a> {
    pub fn new(event: Event, tx: &'a TCPv4TransmitData) -> Self {
        Self {
            completion_token: TCPv4CompletionToken::new(event),
            packet: TCPv4Packet { tx_data: tx },
        }
    }
}

#[repr(C)]
union TCPv4Packet<'a> {
    rx_data: &'a TCPv4ReceiveData<'a>,
    tx_data: &'a TCPv4TransmitData,
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4CompletionToken {
    pub event: Event,
    status: Status,
}

impl TCPv4CompletionToken {
    pub fn new(event: Event) -> Self {
        // PT: Replace in IO with MaybeUninit?
        Self {
            event,
            status: Status::SUCCESS,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4FragmentData {
    fragment_length: u32,
    fragment_buf: *const c_void,
}

impl TCPv4FragmentData {
    fn new(data: &[u8]) -> Self {
        let layout = Layout::from_size_align(data.len(), core::mem::align_of::<u8>()).unwrap();
        let buffer = unsafe { alloc::alloc::alloc(layout) } as *mut u8;

        unsafe {
            copy_nonoverlapping(data.as_ptr(), buffer, data.len());
        }

        Self {
            fragment_length: data.len() as u32,
            fragment_buf: buffer as *const c_void,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4ReceiveData<'a> {
    urgent_flag: bool,
    data_length: u32,
    fragment_count: u32,
    fragment_table: &'a [TCPv4FragmentData],
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4TransmitData {
    push: bool,
    urgent: bool,
    data_length: u32,
    fragment_count: u32,
    fragment_table: [TCPv4FragmentData; 0],
}

impl TCPv4TransmitData {
    pub(crate) fn new(data: &[u8]) -> Self {
        let fragment = TCPv4FragmentData::new(data);
        let fragment_ref = &fragment;
        let size_of_fragment = core::mem::size_of::<TCPv4FragmentData>();
        info!("size of fragment {size_of_fragment}");
        let total_size = core::mem::size_of::<Self>() + size_of_fragment;
        info!("total_size {total_size}");
        let layout = Layout::from_size_align(
            total_size,
            core::mem::align_of::<Self>(),
        ).unwrap();
        let s = unsafe {
            let mut s = alloc::alloc::alloc(layout) as *mut Self;
            (*s).push = true;
            (*s).urgent = false;
            (*s).data_length = data.len() as _;
            (*s).fragment_count = 1;
            copy_nonoverlapping(
                fragment_ref as *const _,
                (*s).fragment_table.as_mut_ptr(),
                size_of_fragment,
            );
            &mut *s

            /*
            let fragment_table_ptr = s.add(1) as *mut TCPv4FragmentData; // Offset by the size of the struct
            core::ptr::write(fragment_table_ptr, fragment);
            &mut *s
             */
        };
        unsafe { core::ptr::read(s) }
        //unsafe {Box::from_raw(s)}

        /*
        let fragment = TCPv4FragmentData::new(data);
        let fragment_table_size = core::mem::size_of::<TCPv4FragmentData>();
        let fragment_table_layout = Layout::from_size_align(
            fragment_table_size,
            core::mem::align_of::<TCPv4FragmentData>()
        ).unwrap();
        let fragment_table = unsafe {
            let ptr = alloc::alloc::alloc(fragment_table_layout) as *mut TCPv4FragmentData;
            ptr.copy_from_nonoverlapping((&fragment).as_ptr(), 1 as usize);
            ptr as *const TCPv4FragmentData as &TCPv4FragmentData
        };
        Self {
            push: false,
            urgent: false,
            data_length: data.len() as u32,
            fragment_count: 1,
            //fragment_table: Box::leak(Box::new([fragment])),
            fragment_table: fragment_table,
        }
         */
    }
}

#[derive(Debug)]
#[repr(C)]
pub enum TCPv4ConnectionState {
    Closed = 0,
    Listen = 1,
    SynSent = 2,
    SynReceived = 3,
    Established = 4,
    FinWait1 = 5,
    FinWait2 = 6,
    Closing = 7,
    TimeWait = 8,
    CloseWait = 9,
    LastAck = 10,
}
