# Example of a Crashing App in Rust

This repository is a how-to guide on debugging a crashing app using lldb and core-dump-handler.

## Prerequisites
This example assumes you have installed the [core-dump-handler](https://github.com/IBM/core-dump-handler/#installing-the-chart) into your kubernetes cluster.

Install the `cdcli` client on your machine. 
Download the latest build from releases https://github.com/IBM/core-dump-handler/releases page.Extract the `cdcli` from the zip folder and place it in a folder that is in your `$PATH`.

## Creating a core dump
To start with you need to generate a core dump. The code in the example-crashing-rust-app project takes care of that. 
example-crashing-rust-app is a normal Rust project with the following release build configuration in the [Cargo.toml](https://github.com/No9/example-crashing-rust-app/blob/main/Cargo.toml#L8). 

```
[profile.release]
debug = true
panic = "abort"
```

The `debug = true` line adds the `-g` flag to the build so the exe will contain symbols to assist with debugging.

While `panic = "abort"` enables panics to generate core dumps so not only can we catch system errors but application logic as well.


Log into your kubernetes cluster and run the prebuilt image in a pod on the server.
This will fail automatically and cause a core dump to be created.
```
kubectl run -i -t crasher --image=quay.io/icdh/example-crashing-rust-app --restart=Never
```

## Locate the image
Now look in your object storage and find the name of the zip file that was created.
e.g. d19ef2ef-35d3-4224-8293-f4f9509868f8-dump-1634327833-crasher-example-crashin-1-11.zip  

Each item in the name breaks down as
* d19ef2ef-35d3-4224-8293-f4f9509868f8 - The guid to ensure the name is unique.
* dump - the type of zip
* 1634327833 - the time the dump occurred
* crasher-example-crashin - the name of the application (N.B this is truncated)
* 1 - The pid of the process 
* 11 - The signal that was sent to the process

## Start Debugging
Now run the cdcli command to start a debugging session. 
As the name of the exe is longer than the OS allows you also need to supply the full name of the exe with the `-e` parameter.

An example would be
```
cdcli -i d19ef2ef-35d3-4224-8293-f4f9509868f8-dump-1634327833-crasher-example-crashin-1-11.zip -i quay.io/icdh/example-crashing-rust-app -e example-crashing-rust-app  
```
You will be presented with the following output.

```
Debugging: example-crashing-rust-app 
Runtime: default 
Namespace: observe
Debug Image: quay.io/icdh/default 
App Image: quay.io/icdh/example-crashing-rust-app
Sending pod config using kubectl
stdout: debugger-06e3166c-f113-4291-81f8-8cf2839942c1
Defaulted container "debug-container" out of: debug-container, core-container
error: unable to upgrade connection: container not found ("debug-container")

Retrying connection...
Defaulted container "debug-container" out of: debug-container, core-container
```
If for some reason the container fails to start the you can kill the session by pressing `CTL-C`

Notice the cdcli will keep retrying to connect to the container if it isn't started yet.

You are now logged into a container on the kubernetes cluster and will see a command prompt.
```
[debugger@debugger-06e3166c-f113-4291-81f8-8cf2839942c1 debug]$ 
```
## Inspect the contents of the debug environment

Now run an `ls` command to see the content of the folder.
```
ls
d19ef2ef-35d3-4224-8293-f4f9509868f8-dump-1634327833-crasher-example-crashin-1-11
d19ef2ef-35d3-4224-8293-f4f9509868f8-dump-1634327833-crasher-example-crashin-1-11.zip  
init.sh  
rundebug.sh
```
You can see the folder containing the core dump and some helper scripts.
The `init.sh` script is used by the system to layout the folder structure and isn't needed for debugging.

Run the `env` command to see that the location of the core file and the executable are available as environment variables.

```
...
S3_BUCKET_NAME=cos-core-dump-store
EXE_LOCATION=/shared/example-crashing-rust-app
PWD=/debug
HOME=/home/debugger
CORE_LOCATION=d19ef2ef-35d3-4224-8293-f4f9509868f8-dump-1634327833-crasher-example-crashin-1-11/d19ef2ef-35d3-4224-8293-f4f9509868f8-dump-1634327833-crasher-example-crashin-1-11.core
...
```
## Start a debugging session
You can now start a debug session by simply running the `rundebug.sh` script.
```
./rundebug.sh
```
You will see the command that is ran and be given the lldb command prompt with the core and the exe preloaded.
```
(lldb) target create "/shared/example-crashing-rust-app" --core "d19ef2ef-35d3-4224-8293-f4f9509868f8-dump-1634327833-crasher-example-crashin-1-11/d19ef2ef-35d3-4224-8293-f4f9509868f8-dump-1634327833-crasher-example-crashin-1-11.core"
Core file '/debug/d19ef2ef-35d3-4224-8293-f4f9509868f8-dump-1634327833-crasher-example-crashin-1-11/d19ef2ef-35d3-4224-8293-f4f9509868f8-dump-1634327833-crasher-example-crashin-1-11.core' (x86_64) was loaded.
(lldb)
```

Now you are ready to start inspecting the core dump.

First you can now look at the backtrace by running the `bt` command
```
bt
thread #1, name = 'example-crashin', stop reason = signal SIGSEGV
    frame #0: 0x00007f29d6a66d39 example-crashing-rust-app`abort + 129
    frame #1: 0x00007f29d6a4fc47 example-crashing-rust-app`panic_abort::__rust_start_panic::abort::hc9ba977db9d5330c at lib.rs:43:17
    frame #2: 0x00007f29d6a4fc26 example-crashing-rust-app`__rust_start_panic at lib.rs:38:5
    frame #3: 0x00007f29d6a4532c example-crashing-rust-app`rust_panic at panicking.rs:670:9
    frame #4: 0x00007f29d6a452cb example-crashing-rust-app`std::panicking::rust_panic_with_hook::hca09fd4c19242a20 at panicking.rs:640:5
    frame #5: 0x00007f29d6a339f4 example-crashing-rust-app`std::panicking::begin_panic::_$u7b$$u7b$closure$u7d$$u7d$::hc735bce12f1e36d5 at panicking.rs:542:9
    frame #6: 0x00007f29d6a339bc example-crashing-rust-app`std::sys_common::backtrace::__rust_end_short_backtrace::ha173c0e3158c9985(f=<unavailable>) at backtrace.rs:141:18
    frame #7: 0x00007f29d6a3105c example-crashing-rust-app`std::panicking::begin_panic::h6197dbc48048c483(msg=(data_ptr = "", length = 4)) at panicking.rs:541:12
   frame #8: 0x00007f29d6a33c88 example-crashing-rust-app`example_crashing_rust_app::bar::h48db1e5d2e4e6220(input=(data_ptr = "hello world\xd6)\U0000007f", length = 11)) at main.rs:17:5
   frame #9: 0x00007f29d6a33c06 example-crashing-rust-app`example_crashing_rust_app::foo::h6f1c5c5323d069a1(input=<unavailable>) at main.rs:12:5
    frame #10: 0x00007f29d6a33bf9 example-crashing-rust-app`example_crashing_rust_app::do_test::ha134fca868990e15 at main.rs:7:5
    frame #11: 0x00007f29d6a33b86 example-crashing-rust-app`example_crashing_rust_app::main::h31e9353150d7f0d7 at main.rs:2:5

```
You could use the long hand
```
thread backtrace all
```

You can see at the start that the program exited with a `SIGSEGV` or segmentation fault raised by the panic in our code.

The call stack represents the order of calls as they were executed before the panic.
Let's select the last call in our logic before the the first panic was raised.
In the example output that would be `frame #8`
Type the command or the line that corresponds to `example-crashing-rust-app example_crashing_rust_app::bar`
```
f 8
```
This is short hand for the following which can also be typed.
```
frame select 8
```
The output of either command will be 
```
frame #8: 0x00007f29d6a33c88 example-crashing-rust-app`example_crashing_rust_app::bar::h48db1e5d2e4e6220(input=(data_ptr = "hello world\xd6)\U0000007f", length = 11)) at main.rs:17:5
```
The output represents the currently selected frame and also shows the values passed.
In this case the value was `input=(data_ptr = "hello world\xd6)\U0000007f", length = 11)`

As the function `bar` doesn't do much lets look at the frame where the string was created.

```
f 10
```

Now you can inspect the variables to see what the inner values of the function was.

```
frame variables
```
or
```
v
```
Both of these commands will show us the value of the `text` variable before it was passed to the function.
```
(alloc::string::String) text = {
  vec = {
    buf = {
      ptr = (pointer = "hello world\xd6)\U0000007f", _marker = core::marker::PhantomData<unsigned char> @ 0x00007ffd76719e00)
      cap = 12
      alloc = {}
    }
    len = 11
  }
}
```

## Integrating the source code
That's great if the code is small and easy to follow but what about more complex scenarios?
If you have access to the code you can configure the debugger to use it and print out the code when you select the frame.

Exit the debugger by typing quit

```
quit
```

Now check out the souce code repository.
```
git clone https://github.com/No9/example-crashing-rust-app.git
```

Now start the debugger 

```
./rundebug
```
And set the source code to your downloaded location
```
settings set -- target.source-map "/app" "/debug/example-crashing-rust-app"
```
N.B. `/app` relates to the `WORKDIR` location in the `Dockerfile`

Now when you move to a frame you also get the related source code.
```
f 10
```
Returns
```
frame #10: 0x00007f29d6a33bf9 example-crashing-rust-app`example_crashing_rust_app::do_test::ha134fca868990e15 at main.rs:7:5
   4   	
   5   	pub fn do_test() -> Result<(), Box<dyn std::error::Error>> {
   6   	    let text = format!("hello {}", "world"); 
-> 7   	    foo(&text.as_str());
   8   	    Ok(())
   9   	}
   10  	
```
With an arrow `->` at line indicating where the next call on the stack was made.
Line 7 on this example.

## Clean up
Now quit the debugger.
```
quit
```

And exit the pod 
```
exit
```

The debugging pod should now be deleted
```
pod "debugger-e2775f05-a5ff-4023-80fc-a14180c3b9e6" deleted
```
## Summary

Well done you've just done a core dump analysis on a Rust application!
You should now be able to understand the benefits of capturing cores as they provide a very easy way to capture issues in environments that aren't easy to access and should also give you the confidence to panic applications when they reach an unknown state rather than trying to make erroneous computations.

