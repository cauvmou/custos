use custos::prelude::*;

/// `AddBuf` will be implemented for all compute devices.<br>
/// Because of `S: Shape`, this trait can be implemented for [`Stack`], which uses fixed size stack allocated arrays.<br>
/// Adding a `D: Device = Self` makes it possible to invoke operations with a `CPU` on, for example, `OpenCL` `Buffer`s (if the device uses unified memory), and `Stack` `Buffer`s.
pub trait AddBuf<T, S: Shape = (), D: Device = Self>: Sized + Device {
    /// This operation perfoms element-wise addition.
    fn add(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, Self, S>;
    // ... you can add more operations if you want to do that.
}

// Host CPU implementation
#[cfg(feature = "cpu")]
impl<T, S: Shape, D: MainMemory> AddBuf<T, S, D> for CPU
where
    T: Copy + std::ops::Add<Output = T>, // you can use the custos::Number trait.
                                         // This trait is implemented for all number types (usize, i16, f32, ...)
{
    fn add(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, Self, S> {
        let len = std::cmp::min(lhs.len(), rhs.len());

        // this returns a previously allocated buffer.
        // You can deactivate the caching behaviour by enabling the "realloc" feature
        // to the custos feature list in the Cargo.toml.
        let mut out = self.retrieve(len, (lhs, rhs));

        // By default, the Buffer dereferences to a slice.
        // Therefore, standard indexing can be used.
        // You can pass a CPU Buffer to a function that takes a slice as a parameter, too.
        // However, the corresponding device needs to implement the
        // `MainMemory` trait (definitely the case for CPU and Stack, and for unified memory devices OpenCL).
        for i in 0..len {
            // indexing Buffers appears to be slower than indexing slices.
            // Therefore, it is recommended to convert the Buffer to a slice before indexing.
            // (e.g. via creating a function that takes slices as parameters, ...).
            out[i] = lhs[i] + rhs[i];
        }
        out
    }
}

// the attribute macro `#[impl_stack]` from the crate `custos-macro`
// can be placed on top of the CPU implementation to automatically
// generate a Stack implementation.
#[cfg(feature = "stack")]
impl<T, S: Shape, D: MainMemory> AddBuf<T, S, D> for Stack
where
    T: Copy + Default + std::ops::Add<Output = T>,
{
    fn add(&self, lhs: &Buffer<T, D, S>, rhs: &Buffer<T, D, S>) -> Buffer<T, Self, S> {
        let mut out = self.retrieve(S::LEN, ()); // this works as well and in this case (Stack), does exactly the same as the line above.

        for i in 0..S::LEN {
            out[i] = lhs[i] + rhs[i];
        }

        out
    }
}

#[cfg(feature = "opencl")]
// OpenCL implementation
impl<T> AddBuf<T> for OpenCL
where
    T: CDatatype, // the custos::CDatatype trait is used to
                  // get the OpenCL C type string for creating generic OpenCL kernels.
{
    fn add(&self, lhs: &CLBuffer<T>, rhs: &CLBuffer<T>) -> CLBuffer<T> {
        // CLBuffer<T> is the same as Buffer<T, OpenCL>
        // generic OpenCL kernel
        let src = format!("
            __kernel void add(__global const {datatype}* lhs, __global const {datatype}* rhs, __global {datatype}* out) 
            {{
                size_t id = get_global_id(0);
                out[id] = lhs[id] + rhs[id];
            }}
        ", datatype=T::as_c_type_str());

        let len = std::cmp::min(lhs.len(), rhs.len());
        let out = self.retrieve::<T, ()>(len, (lhs, rhs));

        // In the background, the kernel is compiled once. After that, it will be reused for every iteration.
        // The cached kernels are released (or freed) when the underlying OpenCL device is dropped.
        // The arguments are specified with a slice of buffers and/or numbers.
        self.launch_kernel(&src, [len, 0, 0], None, &[&lhs, &rhs, &out])
            .unwrap();
        out
    }
}

#[cfg(feature = "cuda")]
// CUDA Implementation
impl<T: CDatatype> AddBuf<T> for CUDA {
    fn add(&self, lhs: &CUBuffer<T>, rhs: &CUBuffer<T>) -> CUBuffer<T> {
        // CUBuffer<T> is the same as Buffer<T, CUDA>
        // generic CUDA kernel
        let src = format!(
            r#"extern "C" __global__ void add({datatype}* lhs, {datatype}* rhs, {datatype}* out, int numElements)
                {{
                    int idx = blockDim.x * blockIdx.x + threadIdx.x;
                    if (idx < numElements) {{
                        out[idx] = lhs[idx] + rhs[idx];
                    }}
                    
                }}
        "#,
            datatype = T::as_c_type_str()
        );

        let len = std::cmp::min(lhs.len(), rhs.len());
        let out = self.retrieve::<T, ()>(len, (lhs, rhs));
        //or: let out = Cache::get::<T, CUDA, 0>(self, len, (lhs, rhs));

        // The kernel is compiled once with nvrtc and is cached too.
        // The arguments are specified with a vector of buffers and/or numbers.
        self.launch_kernel1d(len, &src, "add", &[lhs, rhs, &out, &len])
            .unwrap();
        out
    }
}

/// WGPU implementation
#[cfg(feature = "wgpu")]
impl<T> AddBuf<T> for WGPU {
    fn add(&self, lhs: &Buffer<T, Self>, rhs: &Buffer<T, Self>) -> Buffer<T, Self> {
        let src = format!(
            "@group(0)
            @binding(0)
            var<storage, read_write> a: array<{datatype}>;
            
            @group(0)
            @binding(1)
            var<storage, read_write> b: array<{datatype}>;
    
            @group(0)
            @binding(2)
            var<storage, read_write> out: array<{datatype}>;
            
            
            @compute
            @workgroup_size(1)
            fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
                out[global_id.x] = a[global_id.x] + b[global_id.x];
            }}
            ",
            datatype = std::any::type_name::<T>()
        );

        let mut out = self.retrieve(lhs.len(), (lhs, rhs));
        self.launch_kernel(&src, [lhs.len() as u32, 1, 1], &[lhs, rhs, &mut out]);
        out
    }
}

pub trait AddOp<'a, T, D: Device> {
    fn add(&self, rhs: &Buffer<'a, T, D>) -> Buffer<'a, T, D>;
}

impl<'a, T: CDatatype, D: AddBuf<T>> AddOp<'a, T, D> for Buffer<'a, T, D> {
    #[inline]
    fn add(&self, rhs: &Buffer<'a, T, D>) -> Buffer<'a, T, D> {
        self.device().add(self, rhs)
    }
}

#[allow(dead_code)]
pub struct OwnStruct<'a, T, D: Device> {
    buf: Buffer<'a, T, D>,
}

impl<'a, T, D: Device> OwnStruct<'a, T, D> {
    #[allow(dead_code)]
    // consider using operator overloading for your own type
    #[inline]
    fn add(&self, rhs: &OwnStruct<T, D>) -> Buffer<T, D>
    where
        T: CDatatype,
        D: AddBuf<T>,
    {
        self.buf.device().add(&self.buf, &rhs.buf)
    }

    // general context
    /*#[inline]
    fn operation(&self, rhs: &OwnStruct<T>, other_arg: &T) -> OwnStruct<T> {
        self.buf.device().operation(self, rhs, other_arg)
    }*/

    // ...
}

fn main() -> custos::Result<()> {
    #[cfg(feature = "cpu")]
    {
        let cpu = CPU::new();

        let lhs = Buffer::from((&cpu, [1, 3, 5, 3, 2, 6]));
        let rhs = Buffer::from((&cpu, [-1, -12, -6, 3, 2, -1]));

        let out = cpu.add(&lhs, &rhs);
        assert_eq!(out.read(), vec![0, -9, -1, 6, 4, 5]); // to read a CPU Buffer, you can also call .as_slice() on it.

        // without specifying a device
        let out = lhs.add(&rhs);
        assert_eq!(out.read(), vec![0, -9, -1, 6, 4, 5]);
    }

    #[cfg(feature = "opencl")] // deactivate this block if the feature is disabled
    {
        let cl_device = OpenCL::new(0)?;

        let lhs = Buffer::from((&cl_device, [1, 2, 3, 4, 5, 6]));
        let rhs = Buffer::from((&cl_device, [6, 5, 4, 3, 2, 1]));

        let out = cl_device.add(&lhs, &rhs);
        assert_eq!(out.read(), &[7, 7, 7, 7, 7, 7]);
    }

    #[cfg(feature = "cuda")]
    {
        let cuda_device = CUDA::new(0)?;

        let lhs = Buffer::from((&cuda_device, [1., 2., 3., 4., 5., 6.]));
        let rhs = Buffer::from((&cuda_device, [6., 5., 4., 3., 2., 1.]));

        let out = cuda_device.add(&lhs, &rhs);
        assert_eq!(out.read(), &[7., 7., 7., 7., 7., 7.]);
    }

    #[cfg(feature = "wgpu")]
    {
        let wgpu_device = WGPU::new(wgpu::Backends::all())?;

        let lhs = Buffer::from((&wgpu_device, [1f32, 2., 3., 4., 5., 6.]));
        let rhs = Buffer::from((&wgpu_device, [6., 5., 4., 3., 2., 1.]));

        let out = wgpu_device.add(&lhs, &rhs);
        assert_eq!(out.read(), &[7., 7., 7., 7., 7., 7.]);
    }

    Ok(())
}
