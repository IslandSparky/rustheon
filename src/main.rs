/*Rustheon Raytheon 703 emulator written in Rust

MIT License
Copyright (c) 2023 Darwin Geiselbrecht
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/


const MAX_INST:i32 = 1000;         // max instructions before checking controls 
enum Mode{
    HALT,
    RUN,
    STEP
}
struct Memory {
    core:[i16;32_768]
}
struct Cpu {
    mode:Mode,
    acr:i16,
    ixr:i16,                        // the index register and extension of the acr
    status: u16,                    // status register
    pcr: u16,                       // program counter
    mbr: u16,                       // memory buffer register
    mar: usize,                     // memory address register
    inr: u8,                        // instruction register
    int_req: u16,                   // interrupt request register
    int_act: u16,                   // interrupt active register
    int_enb: u16,                   // interrupt enabled register

}
impl Cpu{                           // create new implementation of Cpu
    fn new() -> Self {
        Cpu {
            mode:Mode::HALT,
            acr:0,
            ixr:0,                                                
            status:0,                      
            pcr: 0,                        
            mbr: 0,                       
            mar: 0,
            inr: 0,
            int_req:0,                        
            int_act: 0,                       
            int_enb:0,   
        }
    }
    // instruction execution loop, broken periodically to update console
    fn execute(&mut self,memory:&mut Memory) {
        let mut inst_counter = 0;               // counter for number instructions before checking console
        'executing: loop {
            match self.mode {
                Mode::HALT => {
                    break 'executing;
                },
                Mode::STEP => {
                    self.decode(memory);
                    break 'executing;
                },
                Mode::RUN => {
                    self.decode(memory);
                },
            }
            inst_counter += 1;
            if inst_counter >= MAX_INST {           // time to return for console/keyboard check?
                println! ("exiting after {} instructions pcr = {} acr = {} mem = {}"
                            ,inst_counter,self.pcr,self.acr,memory.core[self.pcr as usize]);
                break 'executing;
            }
         }
    }
    // first level decoder
    fn decode(&mut self,memory:&mut Memory) {
        self.fetch();                        // fetch instruction into MBR and INR
        if (self.inr & 0xf0) != 0 {
            self.decode_mem_reference(memory);
            return;
        }
        match self.inr {
            0x00 => {self.decode_generic()},
            0x01 => {self.decode_register()},
            0x02 => {self.decode_din()},
            0x03 => {self.decode_dot()},
            0x04 => {self.decode_ixs()},
            0x05 => {self.decode_dxs()},
            0x06 => {self.decode_llb()},
            0x07 => {self.decode_clb()},
            0x08 => {self.decode_skip()},
            0x09 => {self.decode_shift_arith()},
            0x0A => {self.decode_shift_logical()},
               _ => {self.illegal_instruction()}
        }
    }
    fn  decode_mem_reference(&mut self,memory:&mut Memory) {
        self.pcr += 1;
        match self.inr & 0xf0 {
            0x10 => {self.jmp(memory)},
            0x20 => {self.jsx(memory)},
            0x30 => {self.stb(memory)},
            0x40 => {self.cmb(memory)},
            0x50 => {self.ldb(memory)},
            0x50 => {self.ldb(memory)},
            0x60 => {self.stx(memory)},
            0x70 => {self.stw(memory)},
            0x80 => {self.ldw(memory)},
            0x90 => {self.ldx(memory)},
            0xA0 => {self.add(memory)},
            0xB0 => {self.sub(memory)},
            0xC0 => {self.ori(memory)},
            0xD0 => {self.ore(memory)},
            0xE0 => {self.and(memory)},
            0xF0 => {self.cmw(memory)},
            _    => {self.illegal_instruction()}
            } 
    }
    fn decode_generic(&mut self){}
    fn decode_register(&mut self){}
    fn decode_din(&mut self){}
    fn decode_dot(&mut self){}
    fn decode_ixs(&mut self){}
    fn decode_dxs(&mut self){}
    fn decode_llb(&mut self){}
    fn decode_clb(&mut self){}
    fn decode_skip(&mut self){}
    fn decode_shift_arith(&mut self){}
    fn decode_shift_logical(&mut self){}
    fn illegal_instruction(&mut self){}
// These are the memory reference handlers    
    fn jmp(&mut self,memory:&mut Memory){}
    fn jsx(&mut self,memory:&mut Memory){}
    fn stb(&mut self,memory:&mut Memory){}
    fn cmb(&mut self,memory:&mut Memory){}
    fn ldb(&mut self,memory:&mut Memory){}
    fn stx(&mut self,memory:&mut Memory){}
    fn stw(&mut self,memory:&mut Memory){}
    fn ldw(&mut self,memory:&mut Memory){}
    fn ldx(&mut self,memory:&mut Memory){}
    fn add(&mut self,memory:&mut Memory){}
    fn sub(&mut self,memory:&mut Memory){}
    fn ori(&mut self,memory:&mut Memory){}
    fn ore(&mut self,memory:&mut Memory){}
    fn and(&mut self,memory:&mut Memory){}
    fn cmw(&mut self,memory:&mut Memory){}

    fn fetch(&mut self){} 


}

fn main() {
    let mut cpu = Cpu::new();
    let mut memory:Memory= Memory{core:[0i16;32768]};    
    cpu.mode = Mode::RUN;
    cpu.execute(&mut memory);    
    println! ("PCR ={}",cpu.pcr);
    
}    
