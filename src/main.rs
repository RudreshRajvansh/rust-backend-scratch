use libc;

/*
Scope Of this file it creates a Listening NON-BLOCKING socket and binds it to a specific Address and handles error at each stage.
Author - Rudresh Rajvansh
License - MIT 
Date of Creation - 11 July 2026
*/

fn create_socket() -> (bool,i32){
    unsafe{
        //Domain represent the communication space 
        let domain  = libc::AF_INET;
        let socket_type = libc::SOCK_STREAM;
        let protocol = 0;
        let response = libc::socket(domain, socket_type, protocol);
        if response < 0 {
            println!("Error : Socket not created - {}",std::io::Error::last_os_error().raw_os_error());
            return (false,response);
        }
        else {
            //enable non-blocking to make sure 
            let status_NB = enable_nonblocking(response);
            let status_SR = enable_reuseaddr(response);
            if status_NB && status_SR {
            return (true,response);
            }else{
                return (false,response);
            }
        } 
    }
}

fn enable_nonblocking(socket : i32) -> bool{
    unsafe{
        //Enabling Non-Blocking on Socket 
        let get_operation = libc::F_GETFL;
        let flags = libc::fcntl(socket, get_operation, 0);
        if flags < 0 {
            println!("Error : NON-BLOCKING Flag not Applied - {}",std::io::Error::last_os_error().raw_os_error());
            return false;
        }
        //setting flags on socket
        let set_operation = libc::F_SETFL;
        let response = libc::fcntl(socket, set_operation, flags | libc::O_NONBLOCK);
        if response < 0 {
            println!("Error : NON-BLOCKING Flag not Applied - {}",std::io::Error::last_os_error().raw_os_error());
            return false;
        }
        return true;
    }
}

fn enable_reuseaddr(socket : i32) -> bool{
    unsafe{
        //Enabling REUSEADDR OPTION in socket
        let status : i32 = 1;
        let level = libc::SOL_SOCKET;
        let option = libc::SO_REUSEADDR;
        let length = std::mem::size_of::<i32>() as libc::socklen_t;
        let response = libc::setsockopt(socket, level, option, &status as *const i32 as *const libc::c_void, length);
        if response < 0 {
            println!("Error : REUSEADDR not Applied - {}",std::io::Error::last_os_error().raw_os_error());
            return false;
        }
        return true;
    }
}

fn bind_socket(socket : i32) -> bool{
    unsafe{
        let mut myaddr : libc::sockaddr_in = std::mem::zeroed();
        myaddr.sin_family = libc::AF_INET as u16;
        myaddr.sin_port = libc::htons(8080);
        myaddr.sin_addr = libc::in_addr{s_addr : libc::htonl(libc::INADDR_ANY)};
        let length = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
        let response = libc::bind(socket, &myaddr as *const libc::sockaddr_in as *const libc::sockaddr, length);
        if response < 0 {
            return false;
        }else{
            return true;
        }
    }
} 

fn listen_socket(socket : i32) -> bool{
    unsafe{
        let backlog = 1024;
        let response = libc::listen(socket, backlog);
        if response < 0{
            return false;
        }else{
            return true;
        }
    }
}

fn epoll_creation(socket: i32) -> i32{
    unsafe{
        // creation of a epoll fd 
        let epoll_fd = libc::epoll_create1(0); // 0 -> flags 
        if epoll_fd < 0 {
            println!("Error : Epoll Error - {}",std::io::Error::last_os_error().raw_os_error());
            return -1;
        }
        // addition of socket to epoll fd
        let mut clieve : libc::epoll_event = std::mem::zeroed();// dont know why zero eveyrthinme asys struct istnace created 
        clieve.events = libc::EPOLLIN as u32;
        clieve.u64 = socket as u64;

        let status = libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, socket, &mut clieve);    
        if status < 0{
            println!("Error : Epoll Error - {}",std::io::Error::last_os_error().raw_os_error());
            return -1;
        }
        return epoll_fd;
    }
}

fn accepts(socket: i32, sockep: i32) -> i32{
    unsafe{
        let accept_fd = libc::accept(socket, std::ptr::null_mut(), std::ptr::null_mut());
        // adding accept fd to epoll
        let statusNB = enable_nonblocking(accept_fd);
        if statusNB{
            let mut client_event : libc::epoll_event = std::mem::zeroed();
            client_event.events = libc::EPOLLIN as u32;
            client_event.u64 = accept_fd as u64;

            let status = libc::epoll_ctl(sockep, libc::EPOLL_CTL_ADD, accept_fd, &mut client_event);
            if status < 0{
                println!("Error : Epoll Error - {}",std::io::Error::last_os_error().raw_os_error());
                return -1;
            }else{
                return accept_fd;
            }
        }else{
            return -1;
        }
    }
}

fn readandwrite(epollfd : i32, efd : i32) -> i32{
    unsafe{
        let mut buffer = [0u8;1024];
        let status = libc::read(efd, buffer.as_mut_ptr() as *mut libc::c_void, 1024);
        if status < 0{
            println!("Error : Read Error - {}",std::io::Error::last_os_error().raw_os_error());
            return -1;
        }else if status == 0{
            println!("Connection Closed");
            libc::epoll_ctl(epollfd,libc::EPOLL_CTL_DEL,efd,std::ptr::null_mut());
            libc::close(efd);
            return 1;
        }else{
            let write_status = libc::write(efd, buffer.as_ptr() as *const libc::c_void, status as usize);
            if write_status < 0{
                println!("Error : Read Error - {}",std::io::Error::last_os_error().raw_os_error());
                return -1;
            }else{
                println!("Connection Served!!");
                return 1;
            }
        }
    }
}

fn accept_epoll(socket : i32) -> i32{
    unsafe{
        let efd = epoll_creation(socket);
        if efd < 0{
            println!("Error : Epoll Error - {}",std::io::Error::last_os_error().raw_os_error());
            return -1;
        }
        let mut events: [libc::epoll_event; 64] = std::mem::zeroed();
        loop{
            let n = libc::epoll_wait(efd, events.as_mut_ptr(), 64, -1);
            for i in 0..n{
                if events[i as usize].u64 == socket as u64 {
                    let accept_status = accepts(socket, efd);
                    if accept_status < 0{
                        println!("Error : Client Accept Error - {}",std::io::Error::last_os_error().raw_os_error());
                    }
                }else{
                    let readstatus = readandwrite(efd, events[i as usize].u64 as i32);
                    if readstatus < 0 {
                          println!("Error : Client Read Error - {}",std::io::Error::last_os_error().raw_os_error());
                    }
                }
            }
        }
    }
}

fn main(){
    unsafe{
    let (status,socket) = create_socket();
    if status{
        let bind_status = bind_socket(socket);
        if bind_status{
            let listen_status = listen_socket(socket);
            if listen_status {
                let accept_epoll_status = accept_epoll(socket);
                if accept_epoll_status < 0 {
                     println!("Error : Client Accept Error - {}",std::io::Error::last_os_error().raw_os_error());
                }
            }else{
                println!("Error : Status Change Error - {}",std::io::Error::last_os_error().raw_os_error());
            }
        }else{
            println!("Error : Binding Error - {}",std::io::Error::last_os_error().raw_os_error());
        }
    }else{
        println!("Error : Server Error - {}",std::io::Error::last_os_error().raw_os_error());
    }
}
}