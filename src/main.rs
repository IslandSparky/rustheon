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
const EXR_WORD_MASK:u16 = 0xF000;   // word portion of exr registeer in status word
const EXR_BYTE_MASK:u16 = 0xF800;   // byte portion of exr in status word
const ADFNEG:u16 =   0x0400;        // compare negative  flag
const ADFEQL:u16 =   0x0200;        // compare equal flag
const ADFOVF:u16 =   0x0100;        // overflow flag
const ADFGBL:u16 =   0x0080;        // global mode flag


#[derive(Debug)]
enum Mode{
    HALT,
    RUN,
    STEP
}
#[derive(Debug)]
enum ByteSelect{
    LEFT,
    RIGHT,
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
    int_req: u16,                   // interrupt request register   // int 15 (highest) is MSB
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
            //println!("Top of executing loop");
            match self.mode {
                Mode::HALT => {
                    println!{ "Halt instruction encountered"};
                    self.print_registers();
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
        }
    }
    // first level decoder
    fn decode(&mut self,memory:&mut Memory) {
        self.fetch(memory);                         // fetch instruction into MBR and INR
        self.pcr += 1;                              // increment the program counter
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
        //println!("Exiting decode");
    }
    fn  decode_mem_reference(&mut self,memory:&mut Memory) {
        self. compute_word_address();               // form the effective word address in mar
        match self.inr & 0xf0 {                     //    will be overwritten if word address
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
        self.copy_pcr_to_exr();
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
    }
    fn decode_din(&mut self){
        self.din();
    }
    fn decode_dot(&mut self) {
        self.dot();
    }
    fn decode_ixs(&mut self){
        self.ixs();
    }
    fn decode_dxs(&mut self){
        self.dxs();
    }
    fn decode_llb(&mut self){
        self.llb();
     }
    fn decode_clb(&mut self){
        self.clb();
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
    }
    fn illegal_instruction(&mut self){}

// These are the memory reference handlers    
    fn jmp(&mut self,memory:&mut Memory){               // jump 
        self.compute_word_address();
        self.pcr = self.mar as u16;
    }

    fn jsx(&mut self,memory:&mut Memory){               // jump and store index
        self.compute_word_address();
        self.ixr = self.pcr as i16;
        self.pcr = self.mar as u16;
        self.status = self.status | ADFGBL;  // forces global mode
    }

    fn stb(&mut self,memory:&mut Memory){               // store byte
        let left_right = self.compute_byte_address();
        let mut memory_word = memory.core[self.mar];
        match left_right {
            ByteSelect::RIGHT => {
                memory_word = memory_word & ( 0xFF00 as u16)  as i16 | (self.acr & 0x00FF);
            },
            ByteSelect::LEFT  => {
                memory_word = (memory_word & 0x00FF) | self.acr << 8 ;
            }
        }
        memory.core[self.mar] = memory_word;
    }

    fn cmb(&mut self,memory:&mut Memory){               // compare memory byte
        let left_right = self.compute_byte_address();
        let mut memory_word = memory.core[self.mar];
        self.status = self.status & !(ADFGBL | ADFNEG);
        match left_right {
            ByteSelect::RIGHT => {
                if ((self.acr & 0x00FF) as i8) < ((memory_word & 0x00FF) as i8) {
                    self.status = self.status | ADFNEG;
                } else if ((self.acr & 0x00FF) as i8) == ((memory_word & 0x00FF) as i8){
                    self.status = self.status | ADFEQL;
                } 
            },
            ByteSelect::LEFT  => {  
                memory_word = memory_word >> 8;         
                if ((self.acr & 0x00FF) as i8) < ((memory_word & 0x00FF) as i8) {
                    self.status = self.status | ADFNEG;
                } else if ((self.acr & 0x00FF) as i8) == ((memory_word & 0x00FF) as i8){
                    self.status = self.status | ADFEQL;
                } 
            }
        }
    }

    fn ldb(&mut self,memory:&mut Memory){                   // load byte
        let left_right = self.compute_byte_address();
        let mut memory_word = memory.core[self.mar];
        self.acr = self.acr & (0xFF00 as u16) as i16;
        match left_right {
            ByteSelect::RIGHT => {
                self.acr = self.acr | memory_word & 0x00FF;
            },
            ByteSelect::LEFT  => {
                self.acr = self.acr | memory_word >> 8 & 0x00FF;
            }
        }
    }

    fn stx(&mut self,memory:&mut Memory){               // store index
        self.compute_word_address();
        memory.core[self.mar] = self.ixr;        
    }

    fn stw(&mut self,memory:&mut Memory){               // store word
        self.compute_word_address();
        memory.core[self.mar] = self.acr;
    }

    fn ldw(&mut self,memory:&mut Memory){               // load word
        self.compute_word_address();
        self.acr = memory.core[self.mar];
    }

    fn ldx(&mut self,memory:&mut Memory){               // load index
        self.compute_word_address();
        self.ixr = memory.core[self.mar];
    }

    fn add(&mut self,memory:&mut Memory){               // add 
        self.compute_word_address();
        match self.acr.checked_add(memory.core[self.mar]) {
            Some(value) => {
                self.acr = value;
                self.status = self.status & !ADFOVF;
            },
            None           => {
                self.status = self.status | ADFOVF;     // overflow, note and fake results
                self.acr = ( (self.acr as u16) + (memory.core[self.mar] as u16) ) as i16;
            },
        }; 
    }

    fn sub(&mut self,memory:&mut Memory){               // subtract
        self.compute_word_address();
        match self.acr.checked_sub(memory.core[self.mar]) {
            Some(value) => {
                self.acr = value;
                self.status = self.status & !ADFOVF;
            },
            None           => {
                self.status = self.status | ADFOVF;     // overflow, note and fake results
                self.acr = ( (self.acr as u16) - (memory.core[self.mar] as u16) ) as i16;
            },
        }; 
    }

    fn ori(&mut self,memory:&mut Memory){               // inclusive or
        self.compute_word_address();
        self.acr = memory.core[self.mar] | self.acr;
    }

    fn ore(&mut self,memory:&mut Memory){               // exclusive or
        self.compute_word_address();
        self.acr = memory.core[self.mar] ^ self.acr;
    }

    fn and(&mut self,memory:&mut Memory){               // logical and
        self.compute_word_address();
        self.acr = memory.core[self.mar] & self.acr;
    }

    fn cmw(&mut self,memory:&mut Memory){               // compare word
        self.status = self.status & !(ADFEQL | ADFNEG); // clear compare flags for default
        self.compute_word_address();
        if self.acr < memory.core[self.mar]     {
            self.status = self.status | ADFNEG;
        } else if self.acr == memory.core[self.mar]  {
            self.status = self.status | ADFEQL;
        }
    }

 // These are the generic instruction handlers
    fn inr(&mut self){}
    fn enb(&mut self){}
    fn dsb(&mut self){}
    fn slm(&mut self){                                  // set local mode
        self.status = self.status & !ADFGBL;
    }
    fn sgm(&mut self){                                  // set global mode
        self.status = self.status | ADFGBL;      
    }
    fn cex(&mut self){                                  // copy extension to index
        self.ixr =  (self.ixr & 0x07FF) | (self.status as i16 & EXR_BYTE_MASK as i16);
    }
    fn cxe(&mut self){                                  // copy index to extension
        self.status =  (self.status & !EXR_BYTE_MASK) | (self.ixr as u16 & EXR_BYTE_MASK);
    }
    fn sml(&mut self){                                  // set memory lower
        self.status = (self.status & !EXR_BYTE_MASK) | (self.mbr & 0x000F) << 11;
    }
    fn smu(&mut self){                                  // set memory upper
        self.status = ( (self.status & !EXR_BYTE_MASK) | 0x8000 )| (self.mbr & 0x000F) << 11;        
    }
    fn msk(&mut self){                                  // mask interrupts
        self.int_masked = true;
    }
    fn unm(&mut self){                                  // unmask interrupts
        self.int_masked = false;
    }

// These are register instruction handlers
    fn clr(&mut self){                                  // clear accumulator
        self.acr = 0;
    }
    fn cmp(&mut self){                                  // complement accumulator
        self.acr = -self.acr;
    }
    fn inv(&mut self){                                  // invert accumulator
        self.acr = ((self.acr as u16) ^ 0xFFFF) as i16;
    }
    fn cax(&mut self){                                  // copy accumulator to index
        self.ixr = self.acr;
    }
    fn cxa(&mut self){                                  // copy index to accumulator
        self.acr = self.ixr;
    }
// Direct input handler
    fn din(&mut self){}
// Direct output handler
    fn dot(&mut self){}

    fn ixs(&mut self){                                  // increment index and skip >= 0
        self.ixr = self.ixr +  ( self.mbr & 0x00FF) as i16;
        if self.ixr >= 0 {self.pcr += 1}
    }

    fn dxs(&mut self){                                  // decrement index and skip < 0
        self.ixr = self.ixr -  ( self.mbr & 0x00FF) as i16;
        if self.ixr < 0 {self.pcr += 1}
    }

    fn llb(&mut self){                                  // load literal byte
        self.acr = self.acr | (self.mbr & 0x00FF) as i16;
    }
// Compare literal byte handler
    fn clb(&mut self){}

// These are the skip handlers
    fn saz(&mut self){                                  // skip accumulator zero
        if self.acr == 0 {self.pcr += 1}
    }
    fn sap(&mut self){                                  // skip accumulator positive
        if self.acr >= 0 {self.pcr += 1}
    }
    fn sam(&mut self){                                  // skip accumulator negative
        if self.acr < 0 { self.pcr += 1}
    }
    fn sao(&mut self){                                  // skip accumulator odd
        if self.acr & 1 > 0 {self.pcr +=1}
    }
    fn sls(&mut self){                                  // skip on compare less
        if self.status & ADFNEG != 0 {self.pcr += 1}
    }
    fn sxe(&mut self){                                  // skip if index even
        if self.ixr & 1 == 0 {self.pcr += 1}
    }
    fn seq(&mut self){                                  // skip equal
        if self.status & ADFEQL == 0 {self.pcr += 1}
    }
    fn sne(&mut self){                                  // skip not equal
        if self.status & ADFEQL != 0 {self.pcr += 1}
    }
    fn sgr(&mut self){                                  // skip greater
        if (self.status & ADFEQL == 0) & (self.status & ADFNEG == 0 ) {
            self.pcr += 1;
        } 
    }
    fn sle(&mut self){                                  // skip less than or equal
        if (self.status & ADFEQL != 0) | (self.status & ADFNEG != 0 ) {
            self.pcr += 1;
        }
    }
    fn sno(&mut self){                                  // skip no overflow
        if self.status & ADFOVF == 0 { self.pcr += 1}
    }
    fn sse(&mut self){}
    fn ss0(&mut self){}
    fn ss1(&mut self){}
    fn ss2(&mut self){}
    fn ss3(&mut self){}
// These are the shift arithmetic handlers
    fn sra(&mut self){                              // shift right arithmetic
        println!{"in sra"};
       let count = self.mbr & 0x000F; 
       self.acr = self.acr >> count;
    }
    fn sla(&mut self){                              // shift left arithmetic
        let count = self.mbr & 0x000F; 
        self.acr = self.acr << count; 
        // todo check for overflow       
    }
    fn srad(&mut self){}
    fn slad(&mut self){}
// These are the shift logical handlers
    fn srl(&mut self){                                  // shift right logical
        let count = self.mbr & 0x000F; 
        self.acr = ((self.acr  as u16) >> count) as i16;        
    }
    fn  sll(&mut self){                                 // shift left logical
        let count = self.mbr & 0x000F; 
        self.acr = ((self.acr  as u16) << count) as i16;  
    }
    fn srld(&mut self){}
    fn slld(&mut self){}
    fn  src(&mut self){                             // shift right circular
        let count:u32 = (self.mbr & 0x000F) as u32; 
        let mut u_acr: u16 = self.acr  as u16;
        u_acr = u_acr.rotate_right(count);
        self.acr = u_acr as i16;         
    }
    fn  slc(&mut self){                             // shift left circular
        let count:u32 = (self.mbr & 0x000F) as u32; 
        let mut u_acr: u16 = self.acr  as u16;
        u_acr = u_acr.rotate_left(count);
        self.acr = u_acr as i16;  
    }
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


    fn fetch(&mut self,memory:&mut Memory){            // fetch next instruction into mbr and inr
        self.mar = self.pcr as usize;
        self.mbr = memory.core[self.mar] as u16;
        self.inr = ( (self.mbr & 0xFF00) >> 8) as u8;
    }

    fn compute_word_address(&mut self) {                 // form effective word address in MAR
        self.mar = 0;
        self.mar = self.mar | (self.mbr & 0x07FF) as usize;         // get partial address from instruction
        self.mar = self.mar | ( self.status & (EXR_WORD_MASK >> 1) )as usize ;    //if not indexed, we are finishedd
        if (self.mbr & 0x0800) != 0 {                   // indexed instruction
            if (self.status | ADFGBL) != 0 {            // global mode
                self.mar = self.mar & 0x07FF;           // in global, clear out exr portion
            } 
            self.mar = self.mar + (self.ixr as usize);  // todo does ixr add as negative?
        }
    }

    fn compute_byte_address(&mut self) -> ByteSelect{                // form effective word address in MAR
        self.mar = 0;
        let mut byte_flag =  ByteSelect::LEFT; 
        if (self.mbr & 0x0800) == 0 {                   // handlle non-indexed case
            match self.mbr & 0x0001 {
                0x0000 => {byte_flag = ByteSelect::LEFT},
                0x0001 => {byte_flag = ByteSelect::RIGHT},
                     _ => {}
            }
            self.mar =  ( (self.mbr & 0x7ff) as usize) >> 1;
            self.mar = self.mar | ( (self.status & EXR_BYTE_MASK) >> 1) as usize ;    
        } else {                                          // handle indexed case
            self.mar = (self.mbr & 0x07FF) as usize;
            if (self.status & ADFGBL) != 0 {   // local mode - add in exr
                self.mar = self.mar | (self.status & EXR_BYTE_MASK) as usize ; 
            }
            self.mar = self.mar + (self.ixr as usize);
            match self.mar & 0x0001 {
                0x0000 => {byte_flag = ByteSelect::LEFT},
                0x0001 => {byte_flag = ByteSelect::RIGHT},
                     _ => {},
            }
            self.mar = self.mar >> 1;
        }
        byte_flag                                           // return left or right flag
    }

    fn copy_pcr_to_exr (&mut self){                     // copy high 5 bits of pcr to exr
        self.status= ( (self.pcr << 1) & EXR_BYTE_MASK) | (self.status & !EXR_BYTE_MASK);    
    }

    fn print_registers(&mut self){
        println!("PCR = {:04X}  ACR = {:04X}  IXR =    {:04X}",self.pcr,self.acr,self.ixr);
        println!("MBR = {:04X}  MAR = {:04X}  Status = {:04X}",self.mbr,self.mar,self.status);
        println!("Inr = {:02x}  Mode = {:?}",self.inr,self.mode);        
    } 
}

fn main() {
    let mut cpu = Cpu::new();
    let mut memory:Memory= Memory{core:[0i16;32768]};    
    cpu.mode = Mode::RUN;

    cpu.acr = (0x55FF as u16) as i16;
    cpu.ixr = (0x0000 as u16) as i16;
    memory.core[0x0018] = (0x0000 as u16) as i16;
    memory.core[0] = (0x04FF as u16) as i16;
    cpu.execute(&mut memory);
    println!("Memory location 0x18 = {:04x}",memory.core[0x18] );
}    
