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

// masks defining the contexts of the status register
const EXR_MASK:u16 = 0xF800;        // exr registeer in status word
const ADFNEG:u16 =   0x0400;        // compare negative  flag
const ADFEQL:u16 =   0x0200;        // compare equal flag
const ADFOVF:u16 =   0x0100;        // overflow flag
const ADFGBL:u16 =   0x0080;        // global mode flag

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
    int_masked: bool,               // interrupt mask flip/flop

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
            int_masked: true,   
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
    fn decode_generic(&mut self){
        //inr decoded as 00, instruction still in mbr
        let digit2 = self.mbr & 0x00f0;
        if digit2 == 0 {               // halt instruction, don't increment pcr
            self.mode = Mode::HALT;
            return;
        }

        match digit2 {
            0x0010 => {self.inr()},
            0x0020 => {self.enb()},
            0x0030 => {self.dsb()},
            0x0040 => {self.slm()},
            0x0050 => {self.sgm()},
            0x0060 => {self.cex()},
            0x0070 => {self.cxe()},
            0x0080 => {self.sml()},
            0x0090 => {self.smu()},
            0x00A0 => {self.msk()},
            0x00B0 => {self.unm()},
                  _ => {self.illegal_instruction()}
        }
        self.pcr += 1;              // increment pcr for all others
    }
    fn decode_register(&mut self){
        //inr decoded as 01, instruction still in mbr
        let digit2 = self.mbr & 0x00f0;
        match digit2 {
            0x0010 => {self.clr()},
            0x0020 => {self.cmp()},
            0x0030 => {self.inv()},
            0x0040 => {self.cax()},
            0x0050 => {self.cxa()},
                 _ => {self.illegal_instruction()}
            
        }
        self.pcr += 1;
    }
    fn decode_din(&mut self){
        self.din();
        self.pcr += 1;
    }
    fn decode_dot(&mut self) {
        self.dot();
        self.pcr += 1;
    }
    fn decode_ixs(&mut self){
        self.ixs();
        self.pcr += 1;
    }
    fn decode_dxs(&mut self){
        self.dxs();
        self.pcr += 1;
    }
    fn decode_llb(&mut self){
        self.llb();
        self.pcr += 1;
    }
    fn decode_clb(&mut self){
        self.clb();
        self.pcr +=1;
    }
    fn decode_skip(&mut self){
    // inr decoded as 0x08, instruction still in mbr
    let digit2 = self.mbr & 0x00f0;
    match digit2 {
        0x0000 => {self.saz()},
        0x0010 => {self.sap()},
        0x0020 => {self.sam()},
        0x0030 => {self.sao()},
        0x0040 => {self.sls()},
        0x0050 => {self.sxe()},
        0x0060 => {self.seq()},
        0x0070 => {self.sne()},
        0x0080 => {self.sgr()},
        0x0090 => {self.sle()},
        0x00A0 => {self.sno()},
        0x00B0 => {self.sse()},
        0x00C0 => {self.ss0()},
        0x00D0 => {self.ss1()},
        0x00E0 => {self.ss2()},
        0x00F0 => {self.ss3()},
             _ => {self.illegal_instruction()}
        }
        self.pcr += 1;  
   }
    fn decode_shift_arith(&mut self){
        // inr decoded as 0x09, instruction still in mbr
        let digit2 = self.mbr & 0x00f0;
        match digit2 {
            0x0000 => {self.sra()},
            0x0010 => {self.sla()},
            0x0020 => {self.srad()},
            0x0030 => {self.slad()},
                 _ => {self.illegal_instruction()}
        }
        self.pcr += 1;
    }
    fn decode_shift_logical(&mut self){
        // inr decoded as 0x0A, instruction still in mbr
        let digit2 = self.mbr & 0x00f0;
        match digit2 {
            0x0000 => {self.srl()},
            0x0010 => {self.sll()},
            0x0020 => {self.srld()},
            0x0030 => {self.slld()},
            0x0040 => {self.src()},
            0x0050 => {self.slc()},
            0x0060 => {self.srcd()},
            0x0070 => {self.slcd()},
            0x0080 => {self.srll()},
            0x0090 => {self.slll()},
            0x00A0 => {self.srlr()},
            0x00B0 => {self.sllr()},
            0x00C0 => {self.srcl()},
            0x00D0 => {self.slcl()},
            0x00E0 => {self.srcr()},
            0x00F0 => {self.slcr()},
                 _ => {self.illegal_instruction()}
        }
        self.pcr += 1;
    }
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
 // These are the generic instruction handlers
    fn inr(&mut self){}
    fn enb(&mut self){}
    fn dsb(&mut self){}
    fn slm(&mut self){}
    fn sgm(&mut self){}
    fn cex(&mut self){}
    fn cxe(&mut self){}
    fn sml(&mut self){}
    fn smu(&mut self){}
    fn msk(&mut self){                  // mask interrupts
        self.int_masked = true;
    }
    fn unm(&mut self){                  // unmask interrupts
        self.int_masked = false;
    }
// These are register instruction handlers
    fn clr(&mut self){                  // clear accumulator
        self.acr = 0;
    }
    fn cmp(&mut self){                  // complement accumulator
        self.acr = -self.acr;
    }
    fn inv(&mut self){                  // invert accumulator
        self.acr = ((self.acr as u16) ^ 0xFFFF) as i16;
    }
    fn cax(&mut self){                  // copy accumulator to index
        self.ixr = self.acr;
    }
    fn cxa(&mut self){                  // copy index to accumulator
        self.acr = self.ixr;
    }
// Direct input handler
    fn din(&mut self){}
// Direct output handler
    fn dot(&mut self){}
// Increment index and skip handler
    fn ixs(&mut self){}
// Decrement index and skip handler
    fn dxs(&mut self){}
// Load literal byte handler
    fn llb(&mut self){}
// Compare literal byte handler
    fn clb(&mut self){}
// These are the skip handlers
    fn saz(&mut self){                     // skip accumulator zero
        if self.acr == 0 {self.pcr += 1}
    }
    fn sap(&mut self){                      // skip accumulator positive
        if self.acr >= 0 {self.pcr += 1}
    }
    fn sam(&mut self){                      // skip accumulator negative
        if self.acr < 0 { self.pcr += 1}
    }
    fn sao(&mut self){                      // skip accumulator odd
        if self.acr & 1 > 0 {self.pcr +=1}
    }
    fn sls(&mut self){}
    fn sxe(&mut self){                      // skip if index even
        if self.ixr & 1 == 0 {self.pcr += 1}
    }
    fn seq(&mut self){}
    fn sne(&mut self){}
    fn sgr(&mut self){}
    fn sle(&mut self){}
    fn sno(&mut self){}
    fn sse(&mut self){}
    fn ss0(&mut self){}
    fn ss1(&mut self){}
    fn ss2(&mut self){}
    fn ss3(&mut self){}
// These are the shift arithmetic handlers
    fn sra(&mut self){}
    fn sla(&mut self){}
    fn srad(&mut self){}
    fn slad(&mut self){}
// These are the shift logical handlers
    fn srl(&mut self){}
    fn  sll(&mut self){}
    fn srld(&mut self){}
    fn slld(&mut self){}
    fn  src(&mut self){}
    fn  slc(&mut self){}
    fn srcd(&mut self){}
    fn slcd(&mut self){}
    fn srll(&mut self){}
    fn slll(&mut self){}
    fn srlr(&mut self){}
    fn sllr(&mut self){}
    fn srcl(&mut self){}
    fn slcl(&mut self){}
    fn srcr(&mut self){}
    fn slcr(&mut self){}


    fn fetch(&mut self){} 


}

fn main() {
    let mut cpu = Cpu::new();
    let mut memory:Memory= Memory{core:[0i16;32768]};    
    cpu.mode = Mode::RUN;
    cpu.execute(&mut memory);    
    println! ("PCR ={:04x}",cpu.pcr);
  
}    
