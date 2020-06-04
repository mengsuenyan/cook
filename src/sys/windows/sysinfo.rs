//! 获取windows平台系统参数

/// 处理器架构  
#[derive(Clone, Copy)]
pub enum CpuArch {
    AMD64,
    ARM,
    ARM64,
    IA64,
    X86,
    Unknown,
}

#[derive(Clone)]
pub struct CpuInfo {
    logical_core_nums: usize,
    active_logical_core: u64,
    arch: CpuArch,
}

#[allow(non_snake_case)]
#[repr(C)]
struct WinSystemInfo {
    wProcessorArchitecture: u16,
    wReserved: u16,
    dwPageSize: u32,
    lpMinimumApplicationAddress: *mut u8,
    lpMaximumApplicationAddress: *mut u8,
    dwActiveProcessorMask: *mut u32,
    dwNumberOfProcessors: u32,
    dwProcessorType: u32,
    dwAllocationGranularity: u32,
    wProcessorLevel: u16,
    wProcessorRevision: u16,
}

extern "system" {
    fn GetSystemInfo(info: *mut WinSystemInfo) -> std::ffi::c_void;
}

impl CpuInfo {
    pub fn cpu_info() -> CpuInfo {
        let mut cpu= CpuInfo {
            logical_core_nums: 0,
            active_logical_core: 0,
            arch: CpuArch::Unknown,
        };
        
        CpuInfo::system_info(&mut cpu);
        
        cpu
    }
    
    /// 第id个逻辑核心是否激活  
    pub fn is_logical_core_active(&self, logical_core_id: usize) -> Option<bool> {
        if logical_core_id > (self.logical_core_nums - 1) {
            None
        } else {
            let id = logical_core_id as u64;
            Some((self.active_logical_core & (1 << id)) > 0)
        }
    }
    
    fn system_info(cpu: &mut CpuInfo) {
        unsafe {
            let mut info = std::mem::zeroed();
            GetSystemInfo(&mut info);
            
            cpu.logical_core_nums = info.dwNumberOfProcessors as usize;
            cpu.arch = match info.wProcessorArchitecture {
                9 => CpuArch::AMD64,
                5 => CpuArch::ARM,
                12 => CpuArch::ARM64,
                6 => CpuArch::IA64,
                0 => CpuArch::X86,
                _ => CpuArch::Unknown,
            };
            cpu.active_logical_core = info.dwActiveProcessorMask as u64;
        }
    }
    
    /// 获取CPU逻辑核心数  
    pub fn cpu_logical_core_nums() -> usize {
        unsafe {
            let mut info = std::mem::zeroed();
            GetSystemInfo(&mut info);
            info.dwNumberOfProcessors as usize
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn cpu() {
        dbg!("logical cpu nums: {}", super::CpuInfo::cpu_logical_core_nums());
    }
}
