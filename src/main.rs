use libc;

fn main() {
    unsafe{
        let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM,0);
        if fd < 0 {
            println!("Socket error : {}",std::io::Error::last_os_error());
        }else{
            println!("Socket created successfully with fd: {}",fd);
            let opt: i32 = 1;
            let set = libc::setsockopt(fd, libc::SOL_SOCKET,libc::SO_REUSEADDR,&opt as *const i32 as *const libc::c_void, std::mem::size_of::<i32>() as libc::socklen_t);
            if set == 0 {
                println!("Success reuse enabled");
            }else{
                println!("Error occured : {}",std::io::Error::last_os_error());
            }
            let mut myaddr : libc::sockaddr_in = std::mem::zeroed();
                myaddr.sin_family = libc::AF_INET as u16;
                myaddr.sin_port = libc::htons(8080);
                myaddr.sin_addr = libc::in_addr{ s_addr: libc::htonl(libc::INADDR_ANY) };

            let bb = libc::bind(fd, &myaddr as *const libc::sockaddr_in as *const libc::sockaddr, std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t);
                if bb == 0 {
                    println!("Bind successful");
                    let ll = libc::listen(fd, 1024);
                    if ll == 0 {
                        println!("Listening on port 8080");
                        let mut ev : libc::epoll_event = std::mem::zeroed();
                        ev.events = libc::EPOLLIN as u32;
                        ev.u64 = fd as u64;
                        let flags = libc::fcntl(fd, libc::F_GETFL, 0);
                        let _dd = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                        let ep = libc::epoll_create1(0);
                        let oo = libc::epoll_ctl(ep,libc::EPOLL_CTL_ADD,fd, &mut ev);
                        let mut events: [libc::epoll_event; 64] = std::mem::zeroed();
                        loop{
                        let ew = libc::epoll_wait(ep,events.as_mut_ptr(),64,-1);
                        for i in 0..ew{
                            if events[i as usize].u64 == fd as u64 {
                            let aa = libc::accept(fd,std::ptr::null_mut(),std::ptr::null_mut());
                             let cflags = libc::fcntl(aa, libc::F_GETFL, 0);
                             let _cdd = libc::fcntl(aa, libc::F_SETFL, flags | libc::O_NONBLOCK);
                             let mut cev: libc::epoll_event = std::mem::zeroed();
                            cev.events = libc::EPOLLIN as u32;
                            cev.u64 = aa as u64; 
                            libc::epoll_ctl(ep,libc::EPOLL_CTL_ADD,aa, &mut cev);
                            println!("A client connected! {}",aa);
                            }else{
                              let mut buf = [0u8; 1024];
                              let mut rr = libc::read(events[i as usize].u64 as i32, buf.as_mut_ptr() as *mut libc::c_void, 1024);
                              if rr > 0 {
                                let ww = libc::write(events[i as usize].u64 as i32,buf.as_ptr() as *mut libc::c_void,rr as usize);
                              if ww > 0 {
                                println!("Bytes echoed : {}",ww);
                              }else{
                                println!("Error on write, {}",std::io::Error::last_os_error());
                              }
                              }
                              else if rr == 0{
                                libc::epoll_ctl(ep,libc::EPOLL_CTL_DEL,events[i as usize].u64 as i32,std::ptr::null_mut());
                                libc::close(events[i as usize].u64 as i32);
                              }
                              else if rr<0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::EWOULDBLOCK){
                                println!("Error Occured in read");
                              }
                         
                        }
                    }
                    }
                    }
                    else{
                            println!("Listen error : {}",std::io::Error::last_os_error()); 
                    }
                }else{
                    println!("Bind error : {}",std::io::Error::last_os_error());
                }
        }
    }
}

