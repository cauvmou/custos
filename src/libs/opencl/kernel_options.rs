use crate::{matrix::Matrix, number::Number, Error, Node, GenericOCL};

use super::{api::{enqueue_nd_range_kernel, set_kernel_arg}, CL_CACHE, CLCache, cl_device::InternCLDevice};

pub trait KernelArg<'a, T> {
    fn matrix(&'a self) -> Option<&'a Matrix<T>>;
    fn number(&self) -> Option<T>;
}

impl<'a, T: Copy> KernelArg<'a, T> for Matrix<T> {
    fn matrix(&'a self) -> Option<&'a Matrix<T>> {
        Some(self)
    }

    fn number(&self) -> Option<T> {
        None
    }
}

impl<'a, T: Copy> KernelArg<'a, T> for &'a Matrix<T> {
    fn matrix(&self) -> Option<&'a Matrix<T>> {
        Some(self)
    }

    fn number(&self) -> Option<T> {
        None
    }
}

impl<'a, T: Number> KernelArg<'a, T> for T {
    fn matrix(&self) -> Option<&'a Matrix<T>> {
        None
    }

    fn number(&self) -> Option<T> {
        Some(*self)
    }
}

pub struct KernelOptions<'a, T> {
    src: &'a str,
    lhs: &'a Matrix<T>,
    rhs: Option<&'a Matrix<T>>,
    output: Option<Matrix<T>>,
    tensor_args: Vec<(&'a Matrix<T>, usize)>,
    number_args: Vec<(T, usize)>,
    gws: [usize; 3],
    lws: Option<[usize; 3]>,
    offset: Option<[usize; 3]>,
    wd: usize,
    device: InternCLDevice,
}

impl<'a, T: GenericOCL> KernelOptions<'a, T> {
    pub fn new(device: &InternCLDevice, lhs: &'a Matrix<T>, gws: [usize; 3], src: &'a str) -> KernelOptions<'a, T> {
        let wd;
        if gws[0] == 0 {
            panic!("wrong gws")
        } else if gws[1] == 0 {
            wd=1;
        } else if gws[2] == 0 {
            wd=2;
        } else {
            wd=3;
        }
        let tensor_args = vec![(lhs, 0)];
        KernelOptions {
            src,
            lhs,
            rhs: None,
            output: None,
            tensor_args,
            number_args: Vec::new(),
            gws,
            lws: None,
            offset: None,
            wd,
            device: device.clone(),
        }
    }
    pub fn with_lws(&mut self, lws: [usize; 3]) -> &mut KernelOptions<'a, T> {
        self.lws = Some(lws);
        self
    }

    pub fn with_offset(&mut self, offset: [usize; 3]) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_rhs(&mut self, rhs: &'a Matrix<T>) -> &mut KernelOptions<'a, T> {
        self.tensor_args.push((rhs, 1));
        self.rhs = Some(rhs);
        self
    }
    
    pub fn add_arg<A: KernelArg<'a, T>>(&'a mut self, arg: &'a A) -> &mut KernelOptions<'a, T> {
        let idx = self.number_args.len()+self.tensor_args.len();
        
        match arg.number() {
            Some(number) => self.number_args.push((number, idx)),
            None => self.tensor_args.push((arg.matrix().unwrap(), idx)),
        }
        self
    }
    
    pub fn with_output(&mut self, out_dims: (usize, usize)) -> &mut KernelOptions<'a, T> {
        self.output = Some(CLCache::get(self.device.clone(), Node::new(out_dims)));
        self
    }
    pub fn run(&'a mut self) -> Result<Matrix<T>, Error> {
        let kernel = CL_CACHE.with(|cache| cache.borrow_mut().arg_kernel_cache(self.device.clone(), &self.tensor_args, &self.number_args, self.output.as_ref(), self.src.to_string()))?;
               
        for index in 0..self.number_args.len() {
            let arg = self.number_args.get(index).unwrap();
            set_kernel_arg(&kernel, arg.1, &arg.0)
        }

        enqueue_nd_range_kernel(&self.device.get_queue(), &kernel, self.wd, &self.gws, self.lws.as_ref(), self.offset)?;
        

        match &self.output {
            Some(out) => Ok(out.clone()),
            None => Ok(self.lhs.clone()),
        }
    
    }
}
