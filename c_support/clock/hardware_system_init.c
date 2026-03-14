#include "stm32f407xx.h"
#include <stdint.h>
//void stm32f4_Hardware_Init (void);
//>>>############################################
//#if !defined  (HSI_VALUE)   
#ifndef  HSI_VALUE
  #define HSI_VALUE    ((uint32_t)16000000) /*!< Value of the Internal oscillator in Hz*/
#endif /* HSI_VALUE */   
//<<<<<<##########################################

uint32_t SYS_Freq, HCLK_Freq, PCLK1_Freq, PCLK2_Freq;

#if !defined(PLATFORMIO)
void SystemInit(void)				// be called by stm32f40_41xxx.s(Startup)
{
  #if (__FPU_PRESENT == 1) && (__FPU_USED == 1)
    SCB->CPACR |= ((3UL << 10*2)|(3UL << 11*2));  /* set CP10 and CP11 Full Access */
  #endif
}
#endif

//-------- <<< Use Configuration Wizard in Context Menu >>> -----------------
//

/**
         (#) For the STM32F405xx/07xx and STM32F415xx/17xx devices, the maximum
             frequency of the SYSCLK and HCLK is 168 MHz, PCLK2 84 MHz and PCLK1 42 MHz. 
             Depending on the device voltage range, the maximum frequency should
             be adapted accordingly:
 +-------------------------------------------------------------------------------------+
 | Latency       |                HCLK clock frequency (MHz)                           |
 |               |---------------------------------------------------------------------|
 |               | voltage range  | voltage range  | voltage range   | voltage range   |
 |               | 2.7 V - 3.6 V  | 2.4 V - 2.7 V  | 2.1 V - 2.4 V   | 1.8 V - 2.1 V   |
 |---------------|----------------|----------------|-----------------|-----------------|
 |0WS(1CPU cycle)|0 < HCLK <= 30  |0 < HCLK <= 24  |0 < HCLK <= 22   |0 < HCLK <= 20   |
 |---------------|----------------|----------------|-----------------|-----------------|
 |1WS(2CPU cycle)|30 < HCLK <= 60 |24 < HCLK <= 48 |22 < HCLK <= 44  |20 < HCLK <= 40  |
 |---------------|----------------|----------------|-----------------|-----------------|
 |2WS(3CPU cycle)|60 < HCLK <= 90 |48 < HCLK <= 72 |44 < HCLK <= 66  |40 < HCLK <= 60  |
 |---------------|----------------|----------------|-----------------|-----------------|
 |3WS(4CPU cycle)|90 < HCLK <= 120|72 < HCLK <= 96 |66 < HCLK <= 88  |60 < HCLK <= 80  |
 |---------------|----------------|----------------|-----------------|-----------------|
 |4WS(5CPU cycle)|120< HCLK <= 150|96 < HCLK <= 120|88 < HCLK <= 110 |80 < HCLK <= 100 |
 |---------------|----------------|----------------|-----------------|-----------------|
 |5WS(6CPU cycle)|150< HCLK <= 168|120< HCLK <= 144|110 < HCLK <= 132|100 < HCLK <= 120|
 |---------------|----------------|----------------|-----------------|-----------------|
 |6WS(7CPU cycle)|      NA        |144< HCLK <= 168|132 < HCLK <= 154|120 < HCLK <= 140|
 |---------------|----------------|----------------|-----------------|-----------------|
 |7WS(8CPU cycle)|      NA        |      NA        |154 < HCLK <= 168|140 < HCLK <= 160|
 +-------------------------------------------------------------------------------------+
*/
//=========================================================================== Embedded Flash Configuration
//   <h> Flash Access Control Configuration (FLASH_ACR)
//     <o0.0..2> LATENCY: Latency
//       <i> Default: 5 wait states
//                     <0=> 0 wait states
//                     <1=> 1 wait states
//                     <2=> 2 wait states
//                     <3=> 3 wait states
//                     <4=> 4 wait states
//                     <5=> 5 wait states
//                     <6=> 6 wait states
//                     <7=> 7 wait states
//     <o0.8> PRFTBE: Prefetch Buffer Enable
//       <i> Default: enabled
//     <o0.9> ICEN: Instruction cache enable 
//       <i> Default: enabled
//     <o0.10> DCEN: Data cache enable
//       <i> Default: enabled
//   </h>
#define __EFI_ACR_Val             0x00000705
    /* Configure Flash prefetch (Bit(8)), Instruction cache (Bit(9)), Data cache (Bit(10)) and wait state */
//Example Code:    FLASH->ACR = FLASH_ACR_PRFTEN | FLASH_ACR_ICEN |FLASH_ACR_DCEN |FLASH_ACR_LATENCY_5WS;


//=========================================================================== Clock Configuration
// <h> Clock Configuration
//   <h> Clock Control Register Configuration (RCC_CR)
//       <i> Reset value: 0x0000 XX83: HSITRIM[4:0]=0x10; HSION = 1
//     <o0.19> CSSON: Clock Security System enable WHEN HSE oscillator is ready 
//       <i> Default: Clock detector OFF
//     <o0.18> HSEBYP: External High Speed clock Bypass
//       <i> If set, HSE oscillator bypassed with an external clock, when HSEON = 1.
//       <i> HSEBYP be written only if HSE oscillator is disabled.
//     <o0.16> HSEON: External High Speed clock enable 
//       <i> Default: HSE oscillator OFF
//       <i> Cleared by hardware when entering Stop or Standby mode.
//     <o0.3..7> HSITRIM: Internal High Speed clock trimming  <0-31>
//       <i> Default: 0x10=16
//     <o0.0> HSION: Internal High Speed clock enable
//       <i> Default: internal 16MHz RC oscillator ON
//       <i> Set by hardware to force the HSI oscillator ON when leaving Stop or Standby mode
//       <i>     or in case of a failure of the HSE oscillator
//     <e0.24> PLLON: PLL enable (RCC_PLLCFGR)       
//       <i> Default: PLL Disabled
//       <o2.22> PLLSRC: PLL (and audio PLL) entry clock source         
//         <i> Default: HSI
//                       <0=> HSI
//                       <1=> HSE 
//       <o2.0..5> PLLM: Division factor for PLLSRC as PLL input clock (2 ~ 63) <2-63:1>
//         <i> Default: PLLM = 8 for HSE clock 8MHz
//         <i> VCO INPUT (1 ~ 2 MHz) = PLL entry clock source / PLLM 
//         <i> recommended to select 2 MHz for VCO input to limit PLL jitter.
//       <o2.6..14> PLLN: multiplication factor for VCO (192 ~ 432) <192-432>
//         <i> VCO OUTPUT= VCO INPUT x PLLN 
//         <i> 0x150 = 336
//       <o2.16..17> PLLP: division factor for main system clock
//         <i> SYSCLK = PLL output frequency = VCO frequency / PLLP 
//                     <0=>  /2
//                     <1=>  /4
//                     <2=>  /6
//                     <3=>  /8
//       <o2.24..27> PLLQ: division factor for USB OTG FS, SDIO (2 ~ 15) <2-15>
//         <i> The USB OTG FS requires a 48 MHz clock to work correctly. 
//         <i> 336/7 = 48
//     </e>
//   </h>
//   <h> Clock Configuration Register Configuration (RCC_CFGR)
//     <o1.30..31> MCO2: Microcontroller Clock Output 2  
//       <i> Default: MCO2 = SYSCLK
//                     <0=> MCO2_SRC = SYSCLK
//                     <1=> MCO2_SRC = PLLI2S
//                     <2=> MCO2_SRC = HSE
//                     <3=> MCO2_SRC = PLL
//     <o1.21..22> MCO1: Microcontroller Clock Output 1  
//       <i> Default: MCO1 = HSI
//                     <0=> MCO1_SRC = HSI
//                     <1=> MCO1_SRC = LSE
//                     <2=> MCO1_SRC = HSE
//                     <3=> MCO1_SRC = PLL
//     <o1.27..29> MCO2PRE: MCO2 prescaler  
//       <i> Default: no division,i.e., /1
//                     <0=>  MCO2_SRC / 1
//                     <4=>  MCO2_SRC / 2
//                     <5=>  MCO2_SRC / 3
//                     <6=>  MCO2_SRC / 4
//                     <7=>  MCO2_SRC / 5
//     <o1.24..26> MCO1PRE: MCO1 prescaler  
//       <i> Default: no division,i.e., /1
//                     <0=>  MCO1_SRC / 1
//                     <4=>  MCO1_SRC / 2
//                     <5=>  MCO1_SRC / 3
//                     <6=>  MCO1_SRC / 4
//                     <7=>  MCO1_SRC / 5
//     <o1.16..20> RTCPRE: HSE division factor for RTC clock (1 MHz) <0-31:1>
//       <i> 0 or 1 = no clock; other values: e.g., (2 = HSE /2); (31 = HSE /31)
//     <o1.13..15> PPRE2: APB High speed prescaler (APB2) max. 84 MHz
//       <i> Default: PCLK2 = HCLK
//                     <0=> PCLK2 = HCLK
//                     <4=> PCLK2 = HCLK / 2 
//                     <5=> PCLK2 = HCLK / 4 
//                     <6=> PCLK2 = HCLK / 8 
//                     <7=> PCLK2 = HCLK / 16 
//     <o1.10..12> PPRE1: APB Low speed prescaler (APB1)  max. 42 MHz
//       <i> Default: PCLK1 = HCLK
//                     <0=> PCLK1 = HCLK
//                     <4=> PCLK1 = HCLK / 2 
//                     <5=> PCLK1 = HCLK / 4 
//                     <6=> PCLK1 = HCLK / 8 
//                     <7=> PCLK1 = HCLK / 16 
//     <o1.4..7> HPRE: AHB prescaler to get AHB clock frequency (HCLK)
//       <i> Default: HCLK = SYSCLK
//                     <0=> HCLK = SYSCLK
//                     <8=> HCLK = SYSCLK / 2
//                     <9=> HCLK = SYSCLK / 4
//                     <10=> HCLK = SYSCLK / 8
//                     <11=> HCLK = SYSCLK / 16
//                     <12=> HCLK = SYSCLK / 64
//                     <13=> HCLK = SYSCLK / 128
//                     <14=> HCLK = SYSCLK / 256
//                     <15=> HCLK = SYSCLK / 512
//     <o1.0..1> SW: System Clock Switch
//       <i> Default: SYSCLK = HSE
//                     <0=> SYSCLK = HSI
//                     <1=> SYSCLK = HSE
//                     <2=> SYSCLK = PLLCLK
//   </h>
//   <o3.14..15> VOS: Regulator voltage scaling 
//        <i> controls internal voltage regulator output voltage to achieve a trade-off
//        <i> between performance and power consumption 
//        <i> when does not operate at the maximum frequency.
//                     <2=> Scale 2 (max HCLK= 144M)
//                     <3=> Scale 1 (max HCLK= 168M)
//   <o4> HSE: External High Speed Clock [Hz] (4MHz ~ 16MHz) <4000000-26000000>
//   <i> clock value for the used External High Speed Clock.
//   <i> Default: 8000000  (8MHz)
// </h> End of Clock Configuration
#define __RCC_CR_VAL               0x01010081
#define __RCC_CFGR_VAL             0x00009402
#define __RCC_PLLCFGR_VAL          0x07405408
#define __PWR_CR_VOS_VAL           0x0000C000
#define __HSE                      8000000


//=========================================================================== Clock Configuration
//  <o0> PRE_BITS: Preemption Priority Bits 
//         <i> Default: 0 bits
//                       <0=> 0 bit
//                       <1=> 1 bit
//                       <2=> 2 bits
//                       <3=> 3 bits
//                       <4=> 4 bits

#define __PRE_BITS	2

#define PRE_PRI_BOUND	(0x01<<__PRE_BITS)
#define xx__NVIC_PRIO_BITS          4       /*!< STM32F4XX uses 4 Bits for the Priority Levels */
#define SUB_PRI_BOUND	(0x01<< (xx__NVIC_PRIO_BITS - __PRE_BITS))
#define __pre_LeftShift (xx__NVIC_PRIO_BITS-__PRE_BITS)

/*----------------------------------------------------------------------------
 STM32 Get Preemption Priority Bits.
 return the __PRE_BITS
 *----------------------------------------------------------------------------*/
uint32_t stm32_Get_PreempPrioBits (void) {
	  return (__PRE_BITS);  
} // end of stm32_Get_PreempPrioBits

//=========================================================================== System Timer Configuration
// <e0> System Timer (SysTicker) Configuration
//   <o1.2> CLKSOURCE: System Timer clock source selection
//   <i> For stm32, external clock = HCK/8; processor clock = HCK
//                     <0=> external clock (HCK/8)
//                     <1=> processor clock (HCK)
//   <o2> SYSTICK period [100us] <1-2330:10>
//   <i> Set the timer period for System Timer.
//   <i> Default: 1  (100us)
//   <e1.1> TICKINT: System Timer interrupt enabled
//=========================================================================== Clock Configuration
// <h> System Ticker (Timer) Interrupt Priority
//      <o3> Preemption Priority (0=higest) 
//           <i> NOT over (PRE_BITS) bits
//                       <0=> 0 (highest)
//                       <1=> 1 (1 bit)
//                       <2=> 2 (2 bits)
//                       <3=> 3 (2 bits)
//                       <4=> 4 (3 bits)
//                       <5=> 5 (3 bits)
//                       <6=> 6 (3 bits)
//                       <7=> 7 (3 bits)
//                       <8=> 8 (4 bits)
//                       <9=> 9 (4 bits)
//                       <10=> 10 (4 bits)
//                       <11=> 11 (4 bits)
//                       <12=> 12 (4 bits)
//                       <13=> 13 (4 bits)
//                       <14=> 14 (4 bits)
//                       <15=> 15 (4 bits)
//      <o4> SubPriority (0=higest) 
//           <i> NOT over (4-PRE_BITS) bits
//                       <0=> 0 (highest)
//                       <1=> 1 (1 bit)
//                       <2=> 2 (2 bits)
//                       <3=> 3 (2 bits)
//                       <4=> 4 (3 bits)
//                       <5=> 5 (3 bits)
//                       <6=> 6 (3 bits)
//                       <7=> 7 (3 bits)
//                       <8=> 8 (4 bits)
//                       <9=> 9 (4 bits)
//                       <10=> 10 (4 bits)
//                       <11=> 11 (4 bits)
//                       <12=> 12 (4 bits)
//                       <13=> 13 (4 bits)
//                       <14=> 14 (4 bits)
//                       <15=> 15 (4 bits)
// </h> End of System Ticker (Timer) Interrupt Priority
// </e> End of TICKINT: System Timer interrupt enabled
// </e>
#define __SYSTICK_SETUP           0
#define __SYSTICK_CTRL_VAL        0x00000006
#define __SYSTICK_PERIOD          10
#define __SysTick_PREP	0
#define __SysTick_SUBP	0


#if (__SysTick_PREP  < PRE_PRI_BOUND)
 #define Value_SysTick_PREpo   __SysTick_PREP
#else 
 #define Value_SysTick_PREpo   (PRE_PRI_BOUND-1)
#endif

#if (__SysTick_SUBP  < SUB_PRI_BOUND)
 #define Value_SysTick_SUBpo   __SysTick_SUBP
#else 
 #define Value_SysTick_SUBpo   (SUB_PRI_BOUND-1)
#endif

//=========================================================================== USART Configuration
// <e0> USART Configuration
//--------------------------------------------------------------------------- USART1
//   <e1.0> USART1 : USART #1 enable
//     <o4> Baudrate 
//          <9600=>    9600 Baud
//          <14400=>   14400 Baud
//          <19200=>   19200 Baud
//          <28800=>   28800 Baud
//          <38400=>   38400 Baud
//          <56000=>   56000 Baud
//          <57600=>   57600 Baud
//          <115200=>  115200 Baud
//     <o5.12> Data Bits 
//          <0=>       8 Data Bits
//          <1=>       9 Data Bits
//     <o6.12..13> Stop Bits
//          <1=>     0.5 Stop Bit
//          <0=>       1 Stop Bit
//          <3=>     1.5 Stop Bits
//          <2=>       2 Stop Bits
//     <o7.9..10> Parity 
//          <0=>         No Parity
//          <2=>         Even Parity
//          <3=>         Odd Parity
//     <o8.8..9> Flow Control
//          <0=>         None
//          <3=>         Hardware
//     <o9.0..1> Tx, Rx pins 
//          <0=>         TX = PA9, RX = PA10
//          <1=>         TX = PB6, RX = PB7  
//     <o9.2..3> CTS, RTS pins for flow control
//          <0=>         NOT used  
//          <1=>         CTS = PA11, RTS = PA12  
//     <e3.0> USART1 interrupts
//       <o10.4> USART1_CR1.IDLEIE: IDLE Interrupt enable
//       <o10.5> USART1_CR1.RXNEIE: RXNE Interrupt enable
//       <o10.6> USART1_CR1.TCIE: Transmission Complete Interrupt enable
//       <o10.7> USART1_CR1.TXEIE: TXE Interrupt enable
//       <o10.8> USART1_CR1.PEIE: PE Interrupt enable
//       <o11.6> USART1_CR2.LBDIE: LIN Break Detection Interrupt enable
//       <o12.0> USART1_CR3.EIE: Error Interrupt enable
//       <o12.10> USART1_CR3.CTSIE: CTS Interrupt enable
//     </e>
//   </e>

//--------------------------------------------------------------------------- USART2
//   <e1.1> USART2 : USART #2 enable
//     <o13> Baudrate 
//          <9600=>    9600 Baud
//          <14400=>   14400 Baud
//          <19200=>   19200 Baud
//          <28800=>   28800 Baud
//          <38400=>   38400 Baud
//          <56000=>   56000 Baud
//          <57600=>   57600 Baud
//          <115200=>  115200 Baud
//     <o14.12> Data Bits 
//          <0=>       8 Data Bits
//          <1=>       9 Data Bits
//     <o15.12..13> Stop Bits
//          <1=>     0.5 Stop Bit
//          <0=>       1 Stop Bit
//          <3=>     1.5 Stop Bits
//          <2=>       2 Stop Bits
//     <o16.9..10> Parity 
//          <0=>         No Parity
//          <2=>         Even Parity
//          <3=>         Odd Parity
//     <o17.8..9> Flow Control
//          <0=>         None
//          <3=>         Hardware
//     <o18.0..1> Tx, Rx pins 
//          <0=>         TX = PA2, RX = PA3
//          <1=>         TX = PD5, RX = PD6  
//     <o18.2..3> CTS, RTS pins for flow control
//          <0=>         NOT used  
//          <1=>         CTS = PA0, RTS = PA1  
//          <2=>         CTS = PD3, RTS = PD4  
//     <e3.1> USART2 interrupts
//       <o19.4> USART1_CR2.IDLEIE: IDLE Interrupt enable
//       <o19.5> USART1_CR2.RXNEIE: RXNE Interrupt enable
//       <o19.6> USART1_CR2.TCIE: Transmission Complete Interrupt enable
//       <o19.7> USART1_CR2.TXEIE: TXE Interrupt enable
//       <o19.8> USART1_CR2.PEIE: PE Interrupt enable
//       <o20.6> USART1_CR2.LBDIE: LIN Break Detection Interrupt enable
//       <o21.0> USART1_CR2.EIE: Error Interrupt enable
//       <o21.10> USART1_CR2.CTSIE: CTS Interrupt enable
//     </e>
//   </e>

//--------------------------------------------------------------------------- USART3
//   <e1.2> USART3 : USART #3 enable
//     <o22> Baudrate 
//          <9600=>    9600 Baud
//          <14400=> 1440400 Baud
//          <19200=>   19200 Baud
//          <28800=>   28800 Baud
//          <38400=>   38400 Baud
//          <56000=>   56000 Baud
//          <57600=>   57600 Baud
//          <115200=>  115200 Baud
//     <o23.12> Data Bits 
//          <0=>       8 Data Bits
//          <1=>       9 Data Bits
//     <o24.12..13> Stop Bits
//          <1=>     0.5 Stop Bit
//          <0=>       1 Stop Bit
//          <3=>     1.5 Stop Bits
//          <2=>       2 Stop Bits
//     <o25.9..10> Parity 
//          <0=>         No Parity
//          <2=>         Even Parity
//          <3=>         Odd Parity
//     <o26.8..9> Flow Control
//          <0=>         None
//          <3=>         Hardware
//     <o27.0..1> Tx, Rx pins 
//          <0=>         TX = PB10, RX = PB11
//          <1=>         TX = PC10, RX = PC11 
//          <2=>         TX = PD8,  RX = PD9 
//     <o27.2..3> CTS, RTS pins for flow control
//          <0=>         NOT used  
//          <1=>         CTS = PB13, RTS = PB14  
//          <2=>         CTS = PD11, RTS = PD12  
//     <e3.2> USART3 interrupts
//       <o28.4> USART3_CR1.IDLEIE: IDLE Interrupt enable
//       <o28.5> USART3_CR1.RXNEIE: RXNE Interrupt enable
//       <o28.6> USART3_CR1.TCIE: Transmission Complete Interrupt enable
//       <o28.7> USART3_CR1.TXEIE: TXE Interrupt enable
//       <o28.8> USART3_CR1.PEIE: PE Interrupt enable
//       <o29.6> USART3_CR2.LBDIE: LIN Break Detection Interrupt enable
//       <o30.0> USART3_CR3.EIE: Error Interrupt enable
//       <o30.10> USART3_CR3.CTSIE: CTS Interrupt enable
//     </e>
//   </e>
// </e> End of USART Configuration
#define __USART_SETUP             0                       //  0
#define __USART_USED              0x04                    //  1
#define __USART_DETAILS           0x00                    //  2
#define __USART_INTERRUPTS        0x00                    //  3
#define __USART1_BAUDRATE         115200					  //  4
#define __USART1_DATABITS         0x00000000
#define __USART1_STOPBITS         0x00000000
#define __USART1_PARITY           0x00000000
#define __USART1_FLOWCTRL         0x00000000
#define __USART1_REMAP            0x00000000
#define __USART1_CR1              0x00000000
#define __USART1_CR2              0x00000000
#define __USART1_CR3              0x00000000
#define __USART2_BAUDRATE         115200                    // 13
#define __USART2_DATABITS         0x00000000
#define __USART2_STOPBITS         0x00000000
#define __USART2_PARITY           0x00000000
#define __USART2_FLOWCTRL         0x00000000
#define __USART2_REMAP            0x00000000
#define __USART2_CR1              0x00000000
#define __USART2_CR2              0x00000000
#define __USART2_CR3              0x00000000
#define __USART3_BAUDRATE         115200                    // 22
#define __USART3_DATABITS         0x00000000
#define __USART3_STOPBITS         0x00000000
#define __USART3_PARITY           0x00000000
#define __USART3_FLOWCTRL         0x00000000
#define __USART3_REMAP            0x00000001
#define __USART3_CR1              0x00000000
#define __USART3_CR2              0x00000000
#define __USART3_CR3              0x00000000


/**
  * @brief  Returns the SYSCLK frequency
  *        
  * @note   The system frequency computed by this function is not the real 
  *         frequency in the chip. It is calculated based on the predefined 
  *         constant and the selected clock source:
  * @note     If SYSCLK source is HSI, function returns values based on HSI_VALUE(*)
  * @note     If SYSCLK source is HSE, function returns values based on HSE_VALUE(**)
  * @note     If SYSCLK source is PLL, function returns values based on HSE_VALUE(**) 
  *           or HSI_VALUE(*) multiplied/divided by the PLL factors.         
  * @note     (*) HSI_VALUE is a constant defined in stm32f4xx_hal_conf.h file (default value
  *               16 MHz) but the real value may vary depending on the variations
  *               in voltage and temperature.
  * @note     (**) HSE_VALUE is a constant defined in stm32f4xx_hal_conf.h file (default value
  *                25 MHz), user has to ensure that HSE_VALUE is same as the real
  *                frequency of the crystal used. Otherwise, this function may
  *                have wrong result.
  *                  
  * @note   The result of this function could be not correct when using fractional
  *         value for HSE crystal.
  *           
  * @note   This function can be used by the user application to compute the 
  *         baudrate for the communication peripherals or configure other parameters.
  *           
  * @note   Each time SYSCLK changes, this function must be called to update the
  *         right SYSCLK value. Otherwise, any configuration based on this function will be incorrect.
  *         
  *               
  * @param  None
  * @retval SYSCLK frequency
  */
//__WEAK uint32_t HAL_RCC_GetSysClockFreq(void)	// if multiply defined
uint32_t HAL_RCC_GetSysClockFreq(void)
{
  uint32_t pllm = 0, pllvco = 0, pllp = 0;
  uint32_t sysclockfreq = 0;

  /* Get SYSCLK source -------------------------------------------------------*/
  switch (RCC->CFGR & RCC_CFGR_SWS)
  {
    case RCC_CFGR_SWS_HSI:  /* HSI used as system clock source */
         sysclockfreq = HSI_VALUE;
         break;
    case RCC_CFGR_SWS_HSE:  /* HSE used as system clock  source */
         sysclockfreq = __HSE;
// When use the following instruction, adjust the value of External High Speed oscillator (HSE)			
//       sysclockfreq = HSE_VALUE;    //Adjust the value of External High Speed oscillator (HSE)
         break;
    case RCC_CFGR_SWS_PLL:  /* PLL used as system clock  source */
         /* PLL_VCO = (HSE_VALUE or HSI_VALUE / PLLM) * PLLN
            SYSCLK = PLL_VCO / PLLP */
         pllm = RCC->PLLCFGR & RCC_PLLCFGR_PLLM;
         if ( (RCC->PLLCFGR & RCC_PLLCFGR_PLLSRC) != 0)  // bit 22: RCC_PLLCFGR.PLLSRC
         {
          /* 1: HSE used as PLL clock source */
           pllvco = __HSE;
         }
         else
         {
          /* 0: HSI used as PLL clock source */
           pllvco = HSI_VALUE;    
         }
           pllvco = ((pllvco / pllm) * ((RCC->PLLCFGR & RCC_PLLCFGR_PLLN) >> 6));  // bit 6~14: PLLN

         pllp = ((((RCC->PLLCFGR & RCC_PLLCFGR_PLLP) >> 16) + 1 ) *2);  // bit 16~17: PLLP
      
         sysclockfreq = pllvco/pllp;
         break;
    default:
         sysclockfreq = HSI_VALUE;
         break;
  }
  return sysclockfreq;
}

__inline static uint32_t my_getHCLKFreq() {
  uint32_t tmp;

	/*// SysTick_Config(HAL_RCC_GetHCLKFreq() / 1000);  <=== Library Function */
  tmp = (RCC->CFGR & RCC_CFGR_HPRE)>> 4;  // bits 4~7: AHB prescaler: 
	switch(tmp)
	{
		case 8: tmp = 1;  // 2**1 = 2
		        break;
		case 9: tmp = 2; // 2**2 = 4
		        break;
		case 10: tmp = 3; // 2**3 = 8
		        break;
		case 11: tmp = 4; // 2**4 = 16
		        break;
		case 12: tmp = 6; // 2**6 = 64
		        break;
		case 13: tmp = 7; // 2**7 = 128
		        break;
		case 14: tmp = 8; // 2**8 = 256
		        break;
		case 15: tmp = 9; // 2**9 = 512
		        break;
		default: tmp = 0; // 2**0 = 1
		        break;	
	}
//  return HAL_RCC_GetSysClockFreq() >> tmp;   // also in "stm32f4xx_hal_rcc.c"
    return SYS_Freq >> tmp;   // SYS_Freq is calculated out in stm32f4_Hardware_Init
}

static uint32_t my_getPCLKx_Freq(uint32_t tmp) {

	switch(tmp)
	{
		case 4: tmp = 1;  // 2**1 = 2
		        break;
		case 5: tmp = 2; // 2**2 = 4
		        break;
		case 6: tmp = 3; // 2**3 = 8
		        break;
		case 7: tmp = 4; // 2**4 = 16
		        break;
		default: tmp = 0; // 2**0 = 1
		        break;	
	}
//  return HAL_RCC_GetSysClockFreq() >> tmp;   // also in "stm32f4xx_hal_rcc.c"
    return HCLK_Freq >> tmp;   // SYS_Freq is calculated out in stm32f4_Hardware_Init
}

#if __SYSTICK_SETUP
/*----------------------------------------------------------------------------
 STM32 System Timer setup.
 initializes the SysTick register
 *----------------------------------------------------------------------------*/
__inline static void stm32_SysTickSetup (void) {
  uint32_t tickclockFreq, tmp;

/*// SysTick_Config(HAL_RCC_GetHCLKFreq() / 1000);  <=== Library Function */
  tickclockFreq = my_getHCLKFreq();				// HCLK frequency
#if (__SYSTICK_CTRL_VAL & 0x04)	//bit 2: CLKSOURCE = processor clock
#else
  tickclockFreq /= 8;             //bit 2 = 0: CLKSOURCE = external clock
#endif
  // NOTE: tickclockFreq = 0x0A037A00 = 168 000 000
  tmp = __SYSTICK_PERIOD*(tickclockFreq/10000) -1;  // unit  0.1ms
  if (tmp > 0xFFFFFF){
  	  while (1);      /* The reload value is impossible */
  } else {
   
  SysTick->LOAD  = tmp;                                  /* set reload register */

  //>>>>---------------
    tmp = (Value_SysTick_PREpo << __pre_LeftShift)| Value_SysTick_SUBpo;
    SCB->SHP[(15-4)] = (uint8_t)((tmp << (8 - xx__NVIC_PRIO_BITS)) & 0xff);  /* set Priority for Cortex-M  System Interrupts */
  //<<<<---------------
  
  SysTick->CTRL  = __SYSTICK_CTRL_VAL;                          // set clock source and Interrupt enable

  SysTick->VAL   =  0;                                          // clear  the counter
  SysTick->CTRL |= 0x01;                          		// enable the counter
  }
} // end of stm32_SysTickSetup
#endif


#if __USART_SETUP
#define __DIV(__PCLK, __BAUD)       ((__PCLK*25)/(4*__BAUD))
#define __DIVMANT(__PCLK, __BAUD)   (__DIV(__PCLK, __BAUD)/100)
#define __DIVFRAQ(__PCLK, __BAUD)   (((__DIV(__PCLK, __BAUD) - (__DIVMANT(__PCLK, __BAUD) * 100)) * 16 + 50) / 100)
#define __USART_BRR(__PCLK, __BAUD) ((__DIVMANT(__PCLK, __BAUD) << 4)|(__DIVFRAQ(__PCLK, __BAUD) & 0x0F))

/*----------------------------------------------------------------------------
 STM32 USART setup.
 initializes the USARTx register
 *----------------------------------------------------------------------------*/
__inline static void stm32_UsartSetup (void) {

  if (__USART_USED & 0x01) {                                // USART1 used
//      Tx, Rx pins 
   if      ((__USART3_REMAP & 0x03) == 0x00) {             // USART3 no remap
//          0=         TX = PA9, RX = PA10
    RCC->AHB1ENR |=  (1UL << 0);     // enable clock for GPIOA	
    GPIOA->MODER   = (GPIOA->MODER & ~(0x00FFul<<(2*9))) | (0x00AAul<<(2*9)) ;  //Alternate function mode
    GPIOA->OTYPER  &=~(0x03ul<<9);      // push-pull
    GPIOA->OSPEEDR  = (GPIOA->OSPEEDR & ~(0x00FFul<<(2*9))) | (0x00AAul<<(2*9)) ;  //Fast speed
    GPIOA->PUPDR   |= (GPIOA->PUPDR & ~(0x00FFul<<(2*9))) | (0x0055ul<<(2*9)) ;  //pull-up
    GPIOA->AFR[1]   = (GPIOA->AFR[1]& ~(0xFFFFul<<(4*(9-8)))) | (0x7777ul<<(4*(9-8)));
    }
    else if ((__USART3_REMAP & 0x03) == 0x01) {             // USART3 partial remap
//          1=         TX = PB6, RX = PB7  
			RCC->AHB1ENR |=  (1UL << 1);     // enable clock for GPIOB	
			GPIOB->MODER   = (GPIOB->MODER & ~(0x000Ful<<(2*6))) | (0x000Aul<<(2*6)) ;  //Alternate function mode
			GPIOB->OTYPER  &=~(0x03ul<<6);      // push-pull
			GPIOB->OSPEEDR  = (GPIOB->OSPEEDR & ~(0x000Ful<<(2*6))) | (0x000Aul<<(2*6)) ;  //Fast speed
			GPIOB->PUPDR   |= (GPIOB->PUPDR & ~(0x000Ful<<(2*6))) | (0x0005ul<<(2*6)) ;  //pull-up
			GPIOB->AFR[1]   = (GPIOB->AFR[0]& ~(0x00FFul<<(4*(6)))) | (0x0077ul<<(4*(6)));
    }

	 if (__USART1_FLOWCTRL & 0x0300) {                     // HW flow control enabled
//      CTS, RTS pins for flow control
//          0=         NOT used  
		if      ((__USART3_REMAP & 0x0C) == 0x04) {             // USART3 no remap
//          1=         CTS = PA11, RTS = PA12  
			RCC->AHB1ENR |=  (1UL << 0);     // enable clock for GPIOA	
			GPIOA->MODER   = (GPIOA->MODER & ~(0x00FFul<<(2*11))) | (0x00AAul<<(2*11)) ;  //Alternate function mode
			GPIOA->OTYPER  &=~(0x03ul<<11);      // push-pull
			GPIOA->OSPEEDR  = (GPIOA->OSPEEDR & ~(0x00FFul<<(2*11))) | (0x00AAul<<(2*11)) ;  //Fast speed
			GPIOA->PUPDR   |= (GPIOA->PUPDR & ~(0x00FFul<<(2*11))) | (0x0055ul<<(2*11)) ;  //pull-up
			GPIOA->AFR[1]   = (GPIOA->AFR[1]& ~(0xFFFFul<<(4*(11-8)))) | (0x7777ul<<(4*(11-8)));
      }
    }
  
		RCC->APB2ENR |= (RCC_APB2ENR_USART1EN);
    USART1->BRR  = __USART_BRR(PCLK2_Freq, __USART1_BAUDRATE); // set baudrate
    USART1->CR1  = __USART1_DATABITS | __USART1_PARITY;    // set Data bits and Parity
    USART1->CR2  = __USART1_STOPBITS;                       // set Stop bits
    USART1->CR3  = __USART1_FLOWCTRL;                       // Set Flow Control

    USART1->CR1 |= (USART_CR1_RE | USART_CR1_TE);           // RX, TX enable

    if (__USART_INTERRUPTS & 0x01) {                        // interrupts used
      USART1->CR1 |= __USART1_CR1;
      USART1->CR2 |= __USART1_CR2;
      USART1->CR3 |= __USART1_CR3;
      NVIC->ISER[1]  = (1 << (USART1_IRQn & 0x1F));   // enable interrupt
    }

    USART1->CR1 |= USART_CR1_UE;                            // USART enable
  } // end USART1 used
//------------------ USART 2 ------------
  if (__USART_USED & 0x02) {                                // USART2 used
//      Tx, Rx pins 
   if      ((__USART2_REMAP & 0x03) == 0x00) {             // USART3 no remap
//          0=        TX = PA2, RX = PA3
			RCC->AHB1ENR |=  (1UL << 0);     // enable clock for GPIOA	
			GPIOA->MODER   = (GPIOA->MODER & ~(0x00FFul<<(2*2))) | (0x00AAul<<(2*2)) ;  //Alternate function mode
			GPIOA->OTYPER  &=~(0x03ul<<2);      // push-pull
			GPIOA->OSPEEDR  = (GPIOA->OSPEEDR & ~(0x00FFul<<(2*2))) | (0x00AAul<<(2*2)) ;  //Fast speed
			GPIOA->PUPDR   |= (GPIOA->PUPDR & ~(0x00FFul<<(2*2))) | (0x0055ul<<(2*2)) ;  //pull-up
			GPIOA->AFR[0]   = (GPIOA->AFR[0]& ~(0xFFFFul<<(4*(2)))) | (0x7777ul<<(4*(2)));
    }
    else if ((__USART2_REMAP & 0x03) == 0x01) {             // USART3 partial remap
//          1=        TX = PD5, RX = PD6
			RCC->AHB1ENR |=  (1UL << 3);     // enable clock for GPIOD	
			GPIOD->MODER   = (GPIOD->MODER & ~(0x000Ful<<(2*5))) | (0x000Aul<<(2*5)) ;  //Alternate function mode
			GPIOD->OTYPER  &=~(0x03ul<<8);      // push-pull
			GPIOD->OSPEEDR  = (GPIOD->OSPEEDR & ~(0x000Ful<<(2*5))) | (0x000Aul<<(2*5)) ;  //Fast speed
			GPIOD->PUPDR   |= (GPIOD->PUPDR & ~(0x000Ful<<(2*5))) | (0x0005ul<<(2*5)) ;  //pull-up
			GPIOD->AFR[0]   = (GPIOD->AFR[0]& ~(0x00FFul<<(4*(5)))) | (0x0077ul<<(4*(5)));
    }

	 if (__USART2_FLOWCTRL & 0x0300) {                     // HW flow control enabled
//      CTS, RTS pins for flow control
//          0=         NOT used  
		if      ((__USART2_REMAP & 0x0C) == 0x04) {             // USART3 no remap
//          1=         CTS = PA0, RTS = PA1  
			RCC->AHB1ENR |=  (1UL << 0);     // enable clock for GPIOA	
			GPIOA->MODER   = (GPIOA->MODER & ~(0x00FFul<<(2*0))) | (0x00AAul<<(2*0)) ;  //Alternate function mode
			GPIOA->OTYPER  &=~(0x03ul<<0);      // push-pull
			GPIOA->OSPEEDR  = (GPIOA->OSPEEDR & ~(0x00FFul<<(2*0))) | (0x00AAul<<(2*0)) ;  //Fast speed
			GPIOA->PUPDR   |= (GPIOA->PUPDR & ~(0x00FFul<<(2*0))) | (0x0055ul<<(2*0)) ;  //pull-up
			GPIOA->AFR[0]   = (GPIOA->AFR[0]& ~(0xFFFFul<<(4*(0)))) | (0x7777ul<<(4*(0)));
      }
    else if ((__USART2_REMAP & 0x0C) == 0x08) {             // USART3 partial remap
//          2=        CTS = PD3, RTS = PD4  
			RCC->AHB1ENR |=  (1UL << 3);     // enable clock for GPIOD	
			GPIOD->MODER   = (GPIOD->MODER & ~(0x000Ful<<(2*3))) | (0x000Aul<<(2*3)) ;  //Alternate function mode
			GPIOD->OTYPER  &=~(0x03ul<<3);      // push-pull
			GPIOD->OSPEEDR  = (GPIOD->OSPEEDR & ~(0x000Ful<<(2*3))) | (0x000Aul<<(2*3)) ;  //Fast speed
			GPIOD->PUPDR   |= (GPIOD->PUPDR & ~(0x000Ful<<(2*3))) | (0x0005ul<<(2*3)) ;  //pull-up
			GPIOD->AFR[0]   = (GPIOD->AFR[0]& ~(0x00FFul<<(4*(3)))) | (0x0077ul<<(4*(3)));
      }
    }

		RCC->APB1ENR |= (RCC_APB1ENR_USART2EN);
    USART2->BRR  = __USART_BRR(PCLK1_Freq, __USART2_BAUDRATE); // set baudrate
    USART2->CR1  = __USART2_DATABITS | __USART2_PARITY;      // set Data bits and parity
    USART2->CR2  = __USART2_STOPBITS;                       // set Stop bits
    USART2->CR3  = __USART2_FLOWCTRL;                       // Set Flow Control

    USART2->CR1 |= (USART_CR1_RE | USART_CR1_TE);           // RX, TX enable

    if (__USART_INTERRUPTS & 0x02) {                        // interrupts used
      USART2->CR1 |= __USART2_CR1;
      USART2->CR2 |= __USART2_CR2;
      USART2->CR3 |= __USART2_CR3;
      NVIC->ISER[1]  = (1 << (USART2_IRQn & 0x1F));   // enable interrupt
    }

    USART2->CR1 |= USART_CR1_UE;                            // USART enable
  } // end USART2 used

//------------------- USART 3 -----------
  if (__USART_USED & 0x04) {                                // USART3 used
//      Tx, Rx pins 
   if      ((__USART3_REMAP & 0x03) == 0x00) {             // USART3 no remap
//          0=         TX = PB10, RX = PB11
			RCC->AHB1ENR |=  (1UL << 1);     // enable clock for GPIOB	
			GPIOB->MODER   = (GPIOB->MODER & ~(0x000Ful<<(2*10))) | (0x000Aul<<(2*10)) ;  //Alternate function mode
			GPIOB->OTYPER  &=~(0x03ul<<10);      // push-pull
			GPIOB->OSPEEDR  = (GPIOB->OSPEEDR & ~(0x000Ful<<(2*10))) | (0x000Aul<<(2*10)) ;  //Fast speed
			GPIOB->PUPDR   |= (GPIOB->PUPDR & ~(0x000Ful<<(2*10))) | (0x0005ul<<(2*10)) ;  //pull-up
			GPIOB->AFR[1]   = (GPIOB->AFR[1]& ~(0x00FFul<<(4*(10-8)))) | (0x0077ul<<(4*(10-8)));
    }
    else if ((__USART3_REMAP & 0x03) == 0x01) {             // USART3 partial remap
//          1=        TX = PC10, RX = PC11 
			RCC->AHB1ENR |=  (1UL << 2);     // enable clock for GPIOC	
			GPIOC->MODER   = (GPIOC->MODER & ~(0x000Ful<<(2*10))) | (0x000Aul<<(2*10)) ;  //Alternate function mode
			GPIOC->OTYPER  &=~(0x03ul<<10);      // push-pull
			GPIOC->OSPEEDR  = (GPIOC->OSPEEDR & ~(0x000Ful<<(2*10))) | (0x000Aul<<(2*10)) ;  //Fast speed
			GPIOC->PUPDR   |= (GPIOC->PUPDR & ~(0x000Ful<<(2*10))) | (0x0005ul<<(2*10)) ;  //pull-up
			GPIOC->AFR[1]   = (GPIOC->AFR[1]& ~(0x00FFul<<(4*(10-8)))) | (0x0077ul<<(4*(10-8)));
    }
    else {                                                  // USART3 full remap
//         2=         TX = PD8,  RX = PD9 
			RCC->AHB1ENR |=  (1UL << 3);     // enable clock for GPIOD	
			GPIOD->MODER   = (GPIOD->MODER & ~(0x000Ful<<(2*8))) | (0x000Aul<<(2*8)) ;  //Alternate function mode
			GPIOD->OTYPER  &=~(0x03ul<<8);      // push-pull
			GPIOD->OSPEEDR  = (GPIOD->OSPEEDR & ~(0x000Ful<<(2*8))) | (0x000Aul<<(2*8)) ;  //Fast speed
			GPIOD->PUPDR   |= (GPIOD->PUPDR & ~(0x000Ful<<(2*8))) | (0x0005ul<<(2*8)) ;  //pull-up
			GPIOD->AFR[1]   = (GPIOD->AFR[1]& ~(0x00FFul<<(4*(8-8)))) | (0x0077ul<<(4*(8-8)));
      }

	 if (__USART3_FLOWCTRL & 0x0300) {                     // HW flow control enabled
//      CTS, RTS pins for flow control
//          0=         NOT used  
		if      ((__USART3_REMAP & 0x0C) == 0x04) {             // USART3 no remap
//          1=         CTS = PB13, RTS = PB14  
			RCC->AHB1ENR |=  (1UL << 1);     // enable clock for GPIOB	
			GPIOB->MODER   = (GPIOB->MODER & ~(0x000Ful<<(2*13))) | (0x000Aul<<(2*13)) ;  //Alternate function mode
			GPIOB->OTYPER  &=~(0x03ul<<13);      // push-pull
			GPIOB->OSPEEDR  = (GPIOB->OSPEEDR & ~(0x000Ful<<(2*13))) | (0x000Aul<<(2*13)) ;  //Fast speed
			GPIOB->PUPDR   |= (GPIOB->PUPDR & ~(0x000Ful<<(2*13))) | (0x0005ul<<(2*13)) ;  //pull-up
			GPIOB->AFR[1]   = (GPIOB->AFR[1]& ~(0x00FFul<<(4*(13-8)))) | (0x0077ul<<(4*(13-8)));
      }
    else if ((__USART3_REMAP & 0x0C) == 0x08) {             // USART3 partial remap
//          2=         CTS = PD11, RTS = PD12  
			RCC->AHB1ENR |=  (1UL << 3);     // enable clock for GPIOD	
			GPIOD->MODER   = (GPIOD->MODER & ~(0x000Ful<<(2*11))) | (0x000Aul<<(2*11)) ;  //Alternate function mode
			GPIOD->OTYPER  &=~(0x03ul<<11);      // push-pull
			GPIOD->OSPEEDR  = (GPIOD->OSPEEDR & ~(0x000Ful<<(2*11))) | (0x000Aul<<(2*11)) ;  //Fast speed
			GPIOD->PUPDR   |= (GPIOD->PUPDR & ~(0x000Ful<<(2*11))) | (0x0005ul<<(2*11)) ;  //pull-up
			GPIOD->AFR[1]   = (GPIOD->AFR[1]& ~(0x00FFul<<(4*(11-8)))) | (0x0077ul<<(4*(11-8)));
      }
    }

    RCC->APB1ENR |= RCC_APB1ENR_USART3EN;                   // enable clock for USART3
    USART3->BRR  = __USART_BRR(PCLK1_Freq, __USART3_BAUDRATE); // set baudrate
    USART3->CR1  = __USART3_DATABITS | __USART3_PARITY;      // set Data bits and parity
    USART3->CR2  = __USART3_STOPBITS;                       // set Stop bits
    USART3->CR3  = __USART3_FLOWCTRL;                       // Set Flow Control

    USART3->CR1 |= (USART_CR1_RE | USART_CR1_TE);           // RX, TX enable

    if (__USART_INTERRUPTS & 0x04) {                        // interrupts used
      USART3->CR1 |= __USART3_CR1;
      USART3->CR2 |= __USART3_CR2;
      USART3->CR3 |= __USART3_CR3;
      NVIC->ISER[1]  = (1 << (USART3_IRQn & 0x1F));   // enable interrupt
    }

    USART3->CR1 |= USART_CR1_UE;                            // USART enable
  } // end USART3 used


} // end of stm32_UsartSetup

 #include <stdio.h>
__WEAK void printf_Some_Clock_Freqencies(void)
{
		printf("\n\rSystem Freq= %d Hz; HCLK Freq= %d Hz;\n\r", SYS_Freq, HCLK_Freq );
		printf("Peripheral 1 Freq= %d Hz; Peripheral 2 Freq= %d Hz;\n\r", PCLK1_Freq, PCLK2_Freq );
}

#endif		// #if __USART_SETUP
//=======================================

//=========================================================================== Embedded Flash Configuration
//   <q0.0> GPIO: call the extern Function "stm32f4_GPIO_SETUP"
//         <i> Check file "GPIO_Ini.c" for this function
#define __GPIO_SETUP           0

#if __GPIO_SETUP
void stm32f4_GPIO_Init (void);
#endif

//######################################//
//
//         "stm32f4_Hardware_Init()"
//
//######################################//
/*=== Steps 1~3 is to replace HAL_Init() + SystemClock_Config() =========*/
void stm32f4_Hardware_Init (void) {
  uint32_t reg_value;

  /* 1. Configure Flash prefetch, Instruction cache, Data cache and wait state */
    // Flash Configuration should be done before System clock switch (RCC->CFGR.SW) is changed from HSI to others
		// Because the CPU clock frequency is increased.
    FLASH->ACR = __EFI_ACR_Val;

  /* 2.0 Clock Configuration*/
    /* Select regulator voltage output Scale 1 mode */
    RCC->APB1ENR |= RCC_APB1ENR_PWREN;	//bit 28, to configure register group PWR
    PWR->CR |= __PWR_CR_VOS_VAL & (0x03ul<<14);	// bits[14:15]=VOS[0:1]: 2=SCALE2, 3=SCALE1; 1=SCALE3 (not for F407,F417), 

 
  RCC->PLLCFGR = __RCC_PLLCFGR_VAL;                  // set PLL configuration register
  RCC->CFGR = (__RCC_CFGR_VAL & 0xFFFFFFFF);         // set clock configuration register
																											// BUT bits 0~4= [SWS:SW] = 0
  RCC->CR   = __RCC_CR_VAL;                          // set clock control register

  if (__RCC_CR_VAL & RCC_CR_HSION) {                 // if HSI enabled
    while ((RCC->CR & RCC_CR_HSIRDY) == 0);          // Wait for HSIRDY = 1 (HSI is ready)
  }

  if (__RCC_CR_VAL & RCC_CR_HSEON) {                 // if HSE enabled
    while ((RCC->CR & RCC_CR_HSERDY) == 0);          // Wait for HSERDY = 1 (HSE is ready)
  }

  if (__RCC_CR_VAL & RCC_CR_PLLON) {                 // if PLL enabled
    while ((RCC->CR & RCC_CR_PLLRDY) == 0);          // Wait for PLLRDY = 1 (PLL is ready)
  }

	  // Wait till SYSCLK is stabilized (depending on selected clock) 
  //    while ((RCC->CFGR & RCC_CFGR_SWS) != ((__RCC_CFGR_VAL<<2) & RCC_CFGR_SWS));
      while (1) {
      	if ((RCC->CFGR & RCC_CFGR_SWS) == ((__RCC_CFGR_VAL<<2) & RCC_CFGR_SWS)) break;
      }

  /* 2.1 Get some clock frequency */
	//==============================//
	SYS_Freq = HAL_RCC_GetSysClockFreq();
  HCLK_Freq = my_getHCLKFreq();				// HCLK frequency
  reg_value = (RCC->CFGR & RCC_CFGR_PPRE1)>> 10;  // bits 10~12: PCLK1 prescaler: 
	PCLK1_Freq = my_getPCLKx_Freq(reg_value);
  reg_value = (RCC->CFGR & RCC_CFGR_PPRE2)>> 13;  // bits 10~12: PCLK1 prescaler: 
	PCLK2_Freq = my_getPCLKx_Freq(reg_value);
//#############################
/*----------------------------------------------------------------------------
 3.0  setting number of bits for
     STM32 Interrupt Preemption Priority.
 *----------------------------------------------------------------------------*/
  reg_value  =  SCB->AIRCR;                                                   /* read old register configuration    */
  reg_value &= ~(0xFFFF0000 | 0x00000700);             /* clear bits 16~31, 8~10 to change               */
  SCB->AIRCR = reg_value| 
               0x05FA0000 | ((7-__PRE_BITS)<<8);  /* 0x05FA0000: writing this register requires 0x05FA in Field "VECTKEY!� */
 /* 3.1 Enable systick and configure system ticker */
#if __SYSTICK_SETUP
  stm32_SysTickSetup ();
#endif
	
#if __GPIO_SETUP
	stm32f4_GPIO_Init ();
#endif
	
	// The following should stay behind "stm32f4_GPIO_Init()".
	// Otherwise, their own GPIO setting will be ignored. 

#if __USART_SETUP
  stm32_UsartSetup();
	printf_Some_Clock_Freqencies();
#endif
	
}
