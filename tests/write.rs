use custos::{Buffer, WriteBuf, CPU};

#[cfg(any(feature = "cuda", feature = "opencl"))]
use custos::Read;

use custos_macro::stack_cpu_test;

#[stack_cpu_test]
#[test]
fn test_write_cpu() {
    let device = CPU::new();
    let mut buf: Buffer<_, _, custos::Dim1<5>> = Buffer::new(&device, 5);
    device.write(&mut buf, &[1., 2., 3., 4., 5.]);
    assert_eq!(buf.as_slice(), &[1., 2., 3., 4., 5.])
}

#[cfg(feature = "cpu")]
#[test]
fn test_write_buf_cpu() {
    use custos::{Buffer, WriteBuf, CPU};

    let device = CPU::new();

    let mut dst: Buffer<i32, CPU, ()> = Buffer::new(&device, 4);

    let src: Buffer<i32, CPU, ()> = Buffer::from((&device, [1, 2, -5, 4]));

    device.write_buf(&mut dst, &src);

    assert_eq!(dst.read(), [1, 2, -5, 4])
}

#[cfg(feature = "opencl")]
#[test]
fn test_write_buf_cl() -> custos::Result<()> {
    use custos::{Buffer, OpenCL, WriteBuf};

    let device = OpenCL::new(0)?;

    let mut dst: Buffer<i32, _> = Buffer::new(&device, 4);

    let src: Buffer<i32, _> = Buffer::from((&device, [1, 2, -5, 4]));

    device.write_buf(&mut dst, &src);

    assert_eq!(dst.read(), [1, 2, -5, 4]);

    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_write_buf_cu() -> custos::Result<()> {
    use custos::{Buffer, WriteBuf, CUDA};

    let device = CUDA::new(0)?;

    let mut dst: Buffer<i32, _> = Buffer::new(&device, 4);

    let src: Buffer<i32, _> = Buffer::from((&device, [1, 2, -5, 4]));

    device.write_buf(&mut dst, &src);

    assert_eq!(dst.read(), [1, 2, -5, 4]);

    Ok(())
}

#[cfg(feature = "opencl")]
#[test]
fn test_write_cl() -> custos::Result<()> {
    let device = custos::OpenCL::new(0)?;
    let mut buf = Buffer::<_, _>::new(&device, 5);
    device.write(&mut buf, &[1., 2., 3., 4., 5.]);
    assert_eq!(device.read(&buf), vec![1., 2., 3., 4., 5.]);
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_write_cuda() -> custos::Result<()> {
    let device = custos::CUDA::new(0)?;
    let mut buf = Buffer::new(&device, 5);
    device.write(&mut buf, &[1., 2., 3., 4., 5.]);
    assert_eq!(device.read(&buf), vec![1., 2., 3., 4., 5.]);
    Ok(())
}
