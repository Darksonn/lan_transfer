use std::ptr;
use std::io::Error;
use std::ffi::CStr;
use std::net::Ipv6Addr;

pub fn interface_to_scope(iface: &[u8]) -> Result<Option<libc::c_uint>, Error> {
    let ifaddr = IfAddrs::getifaddrs()?;
    for addr in ifaddr.iter() {
        if addr.name().to_bytes() == iface {
            if addr.is_ipv6() {
                return Ok(Some(addr.get_index()?));
            }
        }
    }
    Ok(None)
}

pub struct IfAddrs {
    inner: *mut libc::ifaddrs
}

impl IfAddrs {
    pub fn getifaddrs() -> Result<IfAddrs, Error> {
        let mut ptr = ptr::null_mut();
        unsafe {
            let res = libc::getifaddrs(&mut ptr);
            if res != 0 {
                return Err(Error::last_os_error());
            }
        }
        Ok(Self { inner: ptr })
    }
    pub fn iter(&self) -> IfAddrIter {
        IfAddrIter {
            inner: self.inner,
            phantom: std::marker::PhantomData,
        }
    }
}

impl Drop for IfAddrs {
    fn drop(&mut self) {
        unsafe {
            libc::freeifaddrs(self.inner);
        }
    }
}

pub struct IfAddrIter<'a> {
    inner: *const libc::ifaddrs,
    phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> Iterator for IfAddrIter<'a> {
    type Item = IfAddr<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_null() {
            None
        } else {
            unsafe {
                let res = IfAddr { inner: &*self.inner };
                self.inner = (*self.inner).ifa_next;
                Some(res)
            }
        }
    }
}

pub struct IfAddr<'a> {
    inner: &'a libc::ifaddrs,
}
impl<'a> IfAddr<'a> {
    pub fn name(&self) -> &'a CStr {
        unsafe {
            CStr::from_ptr(self.inner.ifa_name)
        }
    }
    pub fn get_index(&self) -> Result<libc::c_uint, Error> {
        let res = unsafe { libc::if_nametoindex(self.inner.ifa_name) };
        if res == 0 {
            return Err(Error::last_os_error());
        }
        Ok(res)
    }
    pub fn is_ipv6(&self) -> bool {
        let addr = self.inner.ifa_addr;
        if addr.is_null() {
            false
        } else {
            unsafe { *addr }.sa_family == libc::AF_INET6 as libc::sa_family_t
        }
    }
    pub fn get_ipv6_addr(&self) -> Option<Ipv6Addr> {
        let addr = self.inner.ifa_addr;
        if addr.is_null() {
            return None;
        }
        if unsafe { *addr }.sa_family != libc::AF_INET6 as libc::sa_family_t {
            return None;
        }
        let sa = &unsafe { *(addr as *const libc::sockaddr_in6) };
        Some(Ipv6Addr::new(
            ((sa.sin6_addr.s6_addr[0] as u16 & 255) << 8) |
                ((sa.sin6_addr.s6_addr[0] as u16 >> 8) & 255),
            ((sa.sin6_addr.s6_addr[1] as u16 & 255) << 8) |
                ((sa.sin6_addr.s6_addr[1] as u16 >> 8) & 255),
            ((sa.sin6_addr.s6_addr[2] as u16 & 255) << 8) |
                ((sa.sin6_addr.s6_addr[2] as u16 >> 8) & 255),
            ((sa.sin6_addr.s6_addr[3] as u16 & 255) << 8) |
                ((sa.sin6_addr.s6_addr[3] as u16 >> 8) & 255),
            ((sa.sin6_addr.s6_addr[4] as u16 & 255) << 8) |
                ((sa.sin6_addr.s6_addr[4] as u16 >> 8) & 255),
            ((sa.sin6_addr.s6_addr[5] as u16 & 255) << 8) |
                ((sa.sin6_addr.s6_addr[5] as u16 >> 8) & 255),
            ((sa.sin6_addr.s6_addr[6] as u16 & 255) << 8) |
                ((sa.sin6_addr.s6_addr[6] as u16 >> 8) & 255),
            ((sa.sin6_addr.s6_addr[7] as u16 & 255) << 8) |
                ((sa.sin6_addr.s6_addr[7] as u16 >> 8) & 255),
        ))
    }
}
