
//-------- <<<  Menu >>> -----------------
//
//=================================================
#define  RS_bit							 6
#define	 LCD_BASE_sel        0x6C000000
#define	 LCD_BASE        (LCD_BASE_sel | ((0x01UL<<(RS_bit+1))-2))
// e.g., A10 as RS: bit 10 = 0x00000400; bit 11 = 0x00000800 ==> bit 11 - 2 = 0x000007FE
// e.g., A6 as RS: bit 6 = 0x00000040; bit 7 = 0x00000080 ==> bit 7 - 2 = 0x0000007E
//=============================================

//==============================================================
#define __Fonts_select         0x0000001E			//0
#define __default_Font         3			//1
#define __LCD_DIRECT	         1      //2 ==> 1: LANDSCAPE��  2: PORTRAIT
//=============================================

//=================================================
#define USE_resetPIN 0		// 0
#define PNr_reset 2		// 1
#define PIN_rs	5     // 2
#define PNr_backlight 1		// 3
#define PIN_bk	15         // 4
//=============================================

//=================================================
#define PNr_FSMC_set1 3									// 0
#define PinBit_FSMC_set1 0xC733     		// 1
#define PNr_FSMC_set2 4									// 2
#define PinBit_FSMC_set2	0xFF80        // 3
#define PNr_FSMC_set3 6									// 4
#define PinBit_FSMC_set3	0x1000        // 5
#define PNr_FSMC_set4 5									// 6
#define PinBit_FSMC_set4	0x1000        // 7
//=============================================


//=================================================
#define Default_TextColor		0xFFFF
#define Default_BackColor		0x0000
//=============================================

//=================================================
#define Show_DeviceID		0x00
//=============================================

//-------- <<< end  >>> ---------------

/* Includes ------------------------------------------------------------------*/
#include "stm324xg_lcd_sklin.h"
//#define USE_HAL_LL
#ifndef USE_HAL_LL
#include "stm32f4xx_fsmc.h"
#else
#include "stm32f4xx_ll_fsmc.h"
#endif


#ifndef Bit
#define Bit(x) 	(0x01ul<<x)
#endif

/** 
  * @brief LCD default font 
  */ 
#if (__default_Font == 0)
	#define LCD_DEFAULT_FONT         Font8
#endif
#if (__default_Font == 1)
	#define LCD_DEFAULT_FONT         Font12
#endif
#if (__default_Font == 2)
	#define LCD_DEFAULT_FONT         Font16
#endif
#if (__default_Font == 3)
	#define LCD_DEFAULT_FONT         Font20
#endif
#if (__default_Font == 4)
	#define LCD_DEFAULT_FONT         Font24
#endif
	
#define ABS(X)  ((X) > 0 ? (X) : -(X))
#define MAX_POLY_CORNERS   200
#define POLY_Y(Z)          ((int32_t)((Points + Z)->X))
#define POLY_X(Z)          ((int32_t)((Points + Z)->Y))

/* Global variables to set the written text color */
typedef struct 
{ 
  uint16_t TextColor;
  uint16_t BackColor;
  sFONT    *pFont; 
}LCD_DrawPropTypeDef;

static LCD_DrawPropTypeDef DrawProp={ \
			Default_TextColor, Default_BackColor, &LCD_DEFAULT_FONT};

//*** Variables concerning FONT ************
#define	pLCD_Currentfonts DrawProp.pFont
static uint16_t wf, hf;	// font_width, font_height
static uint16_t bytes_f, bytes_wf;	// number of bytes per font, that per width line in a font
volatile static uint16_t lcd_id = 0;

//########################################################
//########################################################
//  DRIVER 													 
//########################################################
//########################################################
#if __LCD_DIRECT==1
  #define  LANDSCAPE			1
	#define  LCD_ROW_NUM    240                //pages in ILI9341
	#define  LCD_COL_NUM    320                //columns in ILI9341
#else
  #define  LANDSCAPE			0
	#define  LCD_ROW_NUM    320                //pages
	#define  LCD_COL_NUM    240                //columns
#endif
	#define  LCD_PIXEL_WIDTH   (LCD_COL_NUM-1)  
	#define  LCD_PIXEL_HEIGHT  (LCD_ROW_NUM-1)   

uint16_t LCD_Pixel_Width(void){ return LCD_PIXEL_WIDTH;}
uint16_t LCD_Pixel_Height(void){ return LCD_PIXEL_HEIGHT;}

typedef struct
{
  volatile uint16_t LCD_REG;			// RS = 0
  volatile uint16_t LCD_RAM;      // RS = 1
} LCD_TypeDef;		


	GPIO_TypeDef *GPIO_backlight, *GPIO_reset;
//#define	 LCD_BASE        ((uint32_t)(0x6C000000 |  ((0x01UL<<(RS_bit+1))-2))
// e.g., A10 as RS: bit 10 = 0x00000400; bit 11 = 0x00000800 ==> bit 11 - 2 = 0x000007FE ==> LCD_BASE = x6C00 07FE
// e.g., A6 as RS: bit 6 = 0x00000040; bit 7 = 0x00000080 ==> bit 7 - 2 = 0x0000007E ==> LCD_BASE = x6C00 007E
#define  LCD             ((LCD_TypeDef *) LCD_BASE)
#define  LCD_turnON_backlight	 (GPIO_backlight->BSRR = Bit(PIN_bk))		// set: output 1
#define  LCD_turnOFF_backlight (GPIO_backlight->BSRR = Bit(PIN_bk)<<16)		// reset: output 0
#define	 LCD_RESET_H  		(GPIO_reset->BSRR = Bit(PIN_rs))
#define	 LCD_RESET_L  		(GPIO_reset->BSRR = Bit(PIN_rs)<<16)
//<<<<<<<<<<<

#define LCD_WriteReg(x)	(LCD->LCD_REG = (uint16_t) x)	//lcd write reg
#define LCD_WriteData(x)	(LCD->LCD_RAM = (uint16_t) x)	//lcd write data
#define LCD_ReadData	((uint16_t) LCD->LCD_RAM)	//lcd read data

/********************************************************************************************************
// Display Inversion On & Off
********************************************************************************************************/
#define ReverseLCD()		LCD_WriteReg(0x21)	// Display Inversion ON (21h)
#define NormalLCD()			LCD_WriteReg(0x20)	// Display Inversion ON (21h)

/***********************************************
  * @brief  Configures the Parallel interface (FSMC) for LCD(Parallel mode)
  * @param  None
  * @retval None
  */
static void LCD_FSMCConfig(void)
{
	FSMC_NORSRAM_InitTypeDef  hsram4_Init;
	FSMC_NORSRAM_TimingTypeDef wr_Timing, w_Timing;
	uint32_t FSMC_Bank1_NEx;
//	#define FSMC_NORSRAM_DEVICE                   FSMC_Bank1
//  #define FSMC_NORSRAM_EXTENDED_DEVICE 					FSMC_Bank1E
		FSMC_Bank1_NEx = ((LCD_BASE_sel>>26) & 0x03 )*2;	// for field of BCTR[(NEx-1)*2]
	// 0= NE1; 2= NE2; 4= NE3; 6= NE4 (=FSMC_NORSRAM_BANK4)
	
  /* Enable FSMC clock */
//  RCC_AHB3PeriphClockCmd(RCC_AHB3Periph_FSMC, ENABLE);
		RCC->AHB3ENR |= Bit(0); //bit 0: RCC_AHB3Periph_FSMC


/*-- FSMC Configuration ------------------------------------------------------*/
/*----------------------- SRAM Bank 1 ----------------------------------------*/
  /* FSMC_Bank1 NEx (x=1,..,4) configuration */
	/* Color LCD configuration ------------------------------------
     LCD (FSMC) configured as follow:
        - Data/Address MUX = Disable
        - Memory Type = SRAM
        - Data Width = 16bit //<<<<<============
        - Write Operation = Enable
        - Extended Mode = Disable
        - Asynchronous Wait = Disable */

//====================================
	hsram4_Init.PageSize  = 0;   // extraneous and useless field
//====================================

	hsram4_Init.NSBank = FSMC_Bank1_NEx; // for field of BCTR[(NEx-1)*2]
  hsram4_Init.DataAddressMux = FSMC_DATA_ADDRESS_MUX_DISABLE;
  hsram4_Init.MemoryType = FSMC_MEMORY_TYPE_SRAM;
  hsram4_Init.MemoryDataWidth = FSMC_NORSRAM_MEM_BUS_WIDTH_16;
  hsram4_Init.BurstAccessMode = FSMC_BURST_ACCESS_MODE_DISABLE;
  hsram4_Init.WaitSignalPolarity = FSMC_WAIT_SIGNAL_POLARITY_LOW;
  hsram4_Init.WrapMode = FSMC_WRAP_MODE_DISABLE;
  hsram4_Init.WaitSignalActive = FSMC_WAIT_TIMING_BEFORE_WS;
  hsram4_Init.WriteOperation = FSMC_WRITE_OPERATION_ENABLE;
  hsram4_Init.WaitSignal = FSMC_WAIT_SIGNAL_DISABLE;
  hsram4_Init.ExtendedMode = FSMC_EXTENDED_MODE_DISABLE;		// 0x00000000U
	//disable: Device->BWTR[Bank] = 0x0FFFFFFFU;  //Device=FSMC_Bank1e,
  hsram4_Init.AsynchronousWait = FSMC_ASYNCHRONOUS_WAIT_DISABLE;
  hsram4_Init.WriteBurst = FSMC_WRITE_BURST_DISABLE;

	wr_Timing.AddressSetupTime = 11;		// templet: 15
  wr_Timing.AddressHoldTime = 0;
  wr_Timing.DataSetupTime = 11;		// templet: 60
  wr_Timing.BusTurnAroundDuration = 0;
  wr_Timing.CLKDivision = 0;
  wr_Timing.DataLatency = 0;
  wr_Timing.AccessMode = FSMC_ACCESS_MODE_A;

  // Initialize SRAM control Interface BCTR[(NEx-1)*2]
  FSMC_NORSRAM_Init(FSMC_Bank1, &hsram4_Init);
	// FSMC_Bank1: FSMC Bank1 registers base address = 0xA0000000UL

  // Initialize SRAM timing Interface BCTR[(NEx-1)*2 + 1]
  FSMC_NORSRAM_Timing_Init(FSMC_Bank1, &wr_Timing, FSMC_Bank1_NEx); 

  if(hsram4_Init.ExtendedMode == FSMC_EXTENDED_MODE_ENABLE)
  {
		w_Timing.AddressSetupTime = 9;
		w_Timing.AddressHoldTime = 0;
		w_Timing.DataSetupTime = 8;
		w_Timing.BusTurnAroundDuration = 0;
		w_Timing.CLKDivision = 0;
		w_Timing.DataLatency = 0;
		w_Timing.AccessMode = FSMC_ACCESS_MODE_A;
  // Initialize SRAM extended mode timing Interface BWTR[(NEx-1)*2]
		FSMC_NORSRAM_Extended_Timing_Init(FSMC_Bank1E, &w_Timing, FSMC_Bank1_NEx,  FSMC_EXTENDED_MODE_ENABLE);  
  } else
	{
		    FSMC_Bank1E->BWTR[FSMC_Bank1_NEx] = 0x0FFFFFFFU;	// reset value
	}
  /* Enable FSMC NOR/SRAM Bank1 */
		__FSMC_NORSRAM_ENABLE(FSMC_Bank1, FSMC_Bank1_NEx); 
		//FSMC_Bank1->BTCR[FSMC_Bank1_NEx] |=  FSMC_BCR1_MBKEN;  // bit0: FSMC_BCR1_MBKEN;
}

/********************************************************************************************************
*  Function: GPIO_outPPhigh				                                                           
*  Object: set GPIO pin as output, push-pull, high speed (100 MHz)
*  Input: PortNum (0, ..., 6); PinNum (0, ..., 15)
*  Output: GPIO_x                                  
********************************************************************************************************/
static GPIO_TypeDef* GPIO_outPPhigh(uint32_t PortNum, uint32_t PinNum)
{
	GPIO_TypeDef *GPIO_x;
	uint32_t y=PinNum;

		RCC->AHB1ENR |=  (1UL << PortNum);     // enable clock for GPIOx
    // __SPI1_CS_PORT = 0: PORTA, 1: PORTB, ..., 10: PORTK
    GPIO_x  = (GPIO_TypeDef *)(AHB1PERIPH_BASE + 0x0400 * PortNum);

    GPIO_x->MODER   = (GPIO_x->MODER & ~Bit((y*2+1)) )  | Bit(y*2); //Output mode (01b)
    GPIO_x->OSPEEDR = (GPIO_x->OSPEEDR |  (0x03ul<<(y*2)) ); //high speed (11b)
    GPIO_x->OTYPER  &= ~Bit(y);                      // push-pull (0b)
    GPIO_x->PUPDR   &= ~(0x03ul<<(y*2));       //NO pull (00b)
  if (y < 8)
    GPIO_x->AFR[0]  &= ~(0x0Ful<<(y*4));      // AF0
  else
    GPIO_x->AFR[1]  &= ~(0x0Ful<<((y-8)*4));  // AF0
	
		return GPIO_x;
}

/********************************************************************************************************
*  Function: GPIO_AF12PPhigh				                                                           
*  Object: set GPIO pin as output, push-pull, high speed (100 MHz)
*  Input: PortNum (0, ..., 6); PinNum (0, ..., 15)
*  Output: none                                  
********************************************************************************************************/
static void GPIO_AF12PPhigh(uint32_t PortNum, uint32_t PinBit)
{
	GPIO_TypeDef *GPIO_x;
	uint32_t temp, chg1= PinBit, chg2=0, chg4L=0, chg4H=0;
	uint32_t fill2=0, fill4L=0, fill4H=0;

		RCC->AHB1ENR |=  (1UL << PortNum);     // enable clock for GPIOx
    // __SPI1_CS_PORT = 0: PORTA, 1: PORTB, ..., 10: PORTK
    GPIO_x  = (GPIO_TypeDef *)(AHB1PERIPH_BASE + 0x0400 * PortNum);

  
	for (temp=0; temp<16; temp++){
			if (PinBit & 0x01){
			   chg2 |= (0x03ul) << (temp*2);
			   fill2 |= (0x02ul) << (temp*2);
			  if (temp <8){
			   chg4L |= (0x0Ful) << (temp*4);
			   fill4L |= (0x0Cul) << (temp*4);
			  }else {
			   chg4H |= (0x0Ful) << ((temp-8)*4);
			   fill4H |= (0x0Cul) << ((temp-8)*4);
			  }				
		  }
			PinBit = PinBit >> 1;
	}

		GPIO_x->MODER   = (GPIO_x->MODER & ~chg2)  | fill2; //AF mode (10b)
    GPIO_x->OSPEEDR = (GPIO_x->OSPEEDR |  chg2 ); //high speed (11b)
    GPIO_x->OTYPER  &= ~chg1;                      // push-pull (0b)
    GPIO_x->PUPDR   &= ~chg2;       //NO pull (00b)
  if (chg4L)
	{
    temp = GPIO_x->AFR[0];
    temp &= ~chg4L;						// clear to 0000b
		GPIO_x->AFR[0] = (temp | fill4L);      // AF12  (1100b)
	}
	if (chg4H)
	{
    temp = GPIO_x->AFR[1];
    temp &= ~chg4H;						// clear to 0000b
		GPIO_x->AFR[1] = (temp | fill4H);      // AF12  (1100b)
	}
}

/********************************************************************************************************
*  Function: delay_ms				                                                           
*  Object: lcd init wait..
*  Input: Num
*  Output: none                                  
*  brief: time = Num * 1ms
********************************************************************************************************/
__WEAK void delay_ms(uint16_t Num)
{
		volatile uint16_t Timer;
		while(Num--)
		{
		 	Timer = 11500;
			while(Timer--); 
		}
}

/********************************************************************************************************
*  Function: LCD_DisplayDevID				                                                           
*  Object: Display LCD device ID on the screen
*  Input: none
*  Output: none                                  
*  brief: none
********************************************************************************************************/
#include <stdio.h>
void LCD_DisplayDevID()
{	
  char p_text[13] = "";
	sprintf(p_text,"LCD ID: %04X",lcd_id);	//LCD ID			 	
	LCD_DisplayStringLineCol(1, 2, p_text);		// line 0, column 2
}	   

/********************************************************************************************************
*  Function: LCD_Reset				                                                           
*  Object: lcd reset control
*  Input: none
*  Output: none                                  
*  brief: none
********************************************************************************************************/
void LCD_Reset(void)
{
#if USE_resetPIN
		LCD_RESET_L;
		delay_ms(150);		// 150 ms
		LCD_RESET_H;
		delay_ms(50);	// 50 ms
#endif
}

uint16_t GRAM_mode;
/********************************************************************************************************
*  Function: LCD_Init				                                                           
*  Object: lcd initialization
*  Input: none
*  Output: none                                  
*  brief: none
********************************************************************************************************/
void LCD_Init(void)
{ 
	volatile uint16_t data1, data2;
		//-- LCD PORT INIT --
//		LCD_PortInit();
		GPIO_backlight = GPIO_outPPhigh(PNr_backlight, PIN_bk);	// backlight pin (LIG)
		if (USE_resetPIN) GPIO_reset = GPIO_outPPhigh(PNr_reset, PIN_rs);  	// reset pin (RST)
		if (PinBit_FSMC_set1)	GPIO_AF12PPhigh(PNr_FSMC_set1, PinBit_FSMC_set1);
		if (PinBit_FSMC_set2)	GPIO_AF12PPhigh(PNr_FSMC_set2, PinBit_FSMC_set2);
		if (PinBit_FSMC_set3)	GPIO_AF12PPhigh(PNr_FSMC_set3, PinBit_FSMC_set3);
		if (PinBit_FSMC_set4)	GPIO_AF12PPhigh(PNr_FSMC_set4, PinBit_FSMC_set4);
/*  GPIOD: AF_FSMC, Speed_100MHz , push pull, no pull-up or pull-down 
	  --------------------------------------------------------------
	  | Pin: PD |   0   1      4         5       8    9   10   14   15
    | GLCD    |  D2  D3  (RD/NOE) (WR/NWE)    D13  D14  D15  D0   D1
	  --------------------------------------------------------------
    GPIOE: AF_FSMC, Speed_100MHz , push pull, no pull-up or pull-down 
	  --------------------------------------------------------------
	  | Pin: PE |    7   8   9  10  11  12  13   14   15
    | GLCD    |   D4  D5  D6  D7  D8  D9  D10  D11  D12
	  --------------------------------------------------------------
   GPIOF: AF_FSMC, Speed_100MHz , push pull, no pull-up or pull-down 
	  --------------------------------------------------------------
	  | Pin : PF |   12        
    | GLCD     |  (RS)    
	             (FSMC_A6)  
	  --------------------------------------------------------------
   GPIOG: AF_FSMC, Speed_100MHz , push pull, no pull-up or pull-down 
	  --------------------------------------------------------------
	  | Pin : PG |       12
    | GLCD     |      (CS)
	                 (FSMC_NE4)
	  --------------------------------------------------------------
*/

	
		LCD_FSMCConfig();

		//-- LCD RESET--
		LCD_Reset();
	// ######### Read ID, just for test ##################
		LCD_WriteReg(0xD3);	// 0xD3: Chip Driver ID
 		data1 = LCD_ReadData;				// dummy		
 		data2 = LCD_ReadData;				// ID Version (0x00)
 		data1 = LCD_ReadData;				// driver ID higher byte	(0x0093)
 		data2 = LCD_ReadData;				// driver IDlower byte(0x0041)		

		lcd_id = (uint16_t)(data1<<8) | data2;
		//-------------- Initial Sequence ---------------
		//************* Start Initial Sequence **********//	
		LCD_WriteReg(0xCF);  // Power control B (CFh)
		LCD_WriteData(0x00); 
		LCD_WriteData(0xC1); 
		LCD_WriteData(0X30); 
		LCD_WriteReg(0xED);  // Power on sequence control (EDh)
		LCD_WriteData(0x64); 
		LCD_WriteData(0x03); 
		LCD_WriteData(0X12); 
		LCD_WriteData(0X81); 
		LCD_WriteReg(0xE8);  // Driver timing control A (E8h)
		LCD_WriteData(0x85); 
		LCD_WriteData(0x10); 
		LCD_WriteData(0x7A); 
		LCD_WriteReg(0xCB);  // Power control A (CBh)
		LCD_WriteData(0x39); 
		LCD_WriteData(0x2C); 
		LCD_WriteData(0x00); 
		LCD_WriteData(0x34); 
		LCD_WriteData(0x02); 
		LCD_WriteReg(0xF7);  
		LCD_WriteData(0x20); 
		LCD_WriteReg(0xEA);  
		LCD_WriteData(0x00); 
		LCD_WriteData(0x00); 
		LCD_WriteReg(0xC0);    // Power Control 1 (C0h)
		LCD_WriteData(0x1B);   //VRH[5:0] 
		LCD_WriteReg(0xC1);    // Power Control 2 (C1h)
		LCD_WriteData(0x01);   //SAP[2:0];BT[3:0] 
		LCD_WriteReg(0xC5);    // VCOM Control 1(C5h)
		LCD_WriteData(0x30); 	 //3F
		LCD_WriteData(0x30); 	 //3C
		LCD_WriteReg(0xC7);    // VCOM Control 2(C7h)
		LCD_WriteData(0XB7); 
		LCD_WriteReg(0x3A);   	// Pixel Format Set (3Ah)
		LCD_WriteData(0x55); 		// 16 bits Format
		LCD_WriteReg(0xB1);   
		LCD_WriteData(0x00);   
		LCD_WriteData(0x1A); 
		LCD_WriteReg(0xB6);    // Display Function Control 
		LCD_WriteData(0x0A); 
		LCD_WriteData(0xA2); 
		LCD_WriteReg(0xF2);    // 3Gamma Function Disable 
		LCD_WriteData(0x00); 
		LCD_WriteReg(0x26);    //Gamma curve selected 
		LCD_WriteData(0x01); 
		LCD_WriteReg(0xE0);    //Set Gamma 
		LCD_WriteData(0x0F); 
		LCD_WriteData(0x2A); 
		LCD_WriteData(0x28); 
		LCD_WriteData(0x08); 
		LCD_WriteData(0x0E); 
		LCD_WriteData(0x08); 
		LCD_WriteData(0x54); 
		LCD_WriteData(0XA9); 
		LCD_WriteData(0x43); 
		LCD_WriteData(0x0A); 
		LCD_WriteData(0x0F); 
		LCD_WriteData(0x00); 
		LCD_WriteData(0x00); 
		LCD_WriteData(0x00); 
		LCD_WriteData(0x00); 		 
		LCD_WriteReg(0XE1);    //Set Gamma 
		LCD_WriteData(0x00); 
		LCD_WriteData(0x15); 
		LCD_WriteData(0x17); 
		LCD_WriteData(0x07); 
		LCD_WriteData(0x11); 
		LCD_WriteData(0x06); 
		LCD_WriteData(0x2B); 
		LCD_WriteData(0x56); 
		LCD_WriteData(0x3C); 
		LCD_WriteData(0x05); 
		LCD_WriteData(0x10); 
		LCD_WriteData(0x0F); 
		LCD_WriteData(0x3F); 
		LCD_WriteData(0x3F); 
		LCD_WriteData(0x0F); 
		LCD_WriteReg(0x11); //Exit Sleep
		delay_ms(120);
		// LCD_WriteReg(0x29); //display on

   	LCD_WriteReg(0x36);  // Memory Access Control (36h)
  #if (LANDSCAPE == 1)
		GRAM_mode = 0xA8;	// bit 7, 5, 3 = 1   
    /* AM=1   (address is updated in vertical writing direction)              */
		LCD_WriteData(GRAM_mode);	// bit 7, 5, 3 = 1  
		// bit 3 (BGR) = 1: (Blue:Green:Red)
		// bit 5 (column and row exchange) = 1
		// bit 6 (column addr. order) = 0   rightward: 0 ==> width 
    // bit 7 (page (row) addr. order) = 1  downward: height ==> 0
		//
		//   --------------------> page (x)
		//   |
		//   |
		//   \/
		//   column (y) 
		//
  #else
		GRAM_mode = 0xC8;	// bit 7, 6, 3 = 1    
    /* AM=0   (address is updated in horizontal writing direction)            */
		LCD_WriteData(GRAM_mode);	// bit 7, 6, 3 = 1  
		// bit 3 (BGR) = 1
		// bit 6 (column addr. order) = 1   upward: width ==> 0
    // bit 7 (page (row) addr. order) = 1  rightward: height ==> 0
		//
		//   column (x) 
		//   ^
		//   |
		//   |
		//   --------------------> page (y)
		//
  #endif
	
	//----- setting the following initial variables:
	//----- Current font
	//----- wf, hf, bytes_wf, bytes_f
	LCD_SetFont( &LCD_DEFAULT_FONT);
	//----- text color; background color
	LCD_SetColors(Default_TextColor, Default_BackColor);
	
		LCD_Clear(Default_BackColor);
    delay_ms(50); 
#if Show_DeviceID
		LCD_DisplayDevID();
#endif
		LCD_DisplayOn();
}

/********************************************************************************************************
*  Function: LCD_OpenWin				                                                           
*  Object: lcd open window for display
*  Input: x0,y0, x1, y1
*  Output: none                                  
*  brief: none
********************************************************************************************************/
static void LCD_OpenWin(uint16_t x0, uint16_t y0, uint16_t x1, uint16_t y1)
{
		LCD_WriteReg(0x2A);					// Column Address Set (2Ah)
		LCD_WriteData(x0>>8);
		LCD_WriteData(0x00FF&x0);		
		LCD_WriteData(x1>>8);
		LCD_WriteData(0x00FF&x1);
	
		LCD_WriteReg(0x2B);				// Page (Row) Address Set (2Bh) 
		LCD_WriteData(y0>>8);
		LCD_WriteData(0x00FF&y0);		
		LCD_WriteData(y1>>8);
		LCD_WriteData(0x00FF&y1);

		LCD_WriteReg(0x2C);
}

/********************************************************************************************************
*  Function: LCD_toPortrait				                                                           
*  Object: lcd display in Portrait mode
*  Input: none
*  Output: none                                  
*  brief: none
********************************************************************************************************/
void LCD_toPortrait(void)
{
   	LCD_WriteReg(0x36);  // Memory Access Control (36h)
		LCD_WriteData(0xC8);	// bit 7, 6, 3 = 1  
}
/********************************************************************************************************
*  Function: LCD_toLandscape				                                                           
*  Object: lcd display in Portrait mode
*  Input: none
*  Output: none                                  
*  brief: none
********************************************************************************************************/
void LCD_toLandscape(void)
{
   	LCD_WriteReg(0x36);  // Memory Access Control (36h)
		LCD_WriteData(0xA8);	// bit 7, 6, 3 = 1  
}
/********************************************************************************************************
*  Function: LCD_toDefaultDisDirection				                                                           
*  Object: lcd display in Portrait mode
*  Input: none
*  Output: none                                  
*  brief: none
********************************************************************************************************/
void LCD_toDefaultDisDirection(void)
{
   	LCD_WriteReg(0x36);  // Memory Access Control (36h)
		LCD_WriteData(GRAM_mode);	// bit 7, 6, 3 = 1  
}
/********************************************************************************************************
*  Function: LCD_Clear				                                                           
*  Object: lcd clear screen
*  Input: backcolor
*  Output: none                                  
*  brief: none
********************************************************************************************************/
void LCD_Clear(uint16_t BackColor)
{
		uint16_t i,j;
		LCD_OpenWin(0, 0, LCD_COL_NUM-1, LCD_ROW_NUM-1);
		for(i = 0; i < LCD_ROW_NUM; i++)
			 for(j =0; j < LCD_COL_NUM; j++)
					 LCD_WriteData(BackColor);
}

/**
  * @brief  Enable the Display.
  * @param  None
  * @retval None
  */
void LCD_DisplayOn(void)
{
 	//---- turn on backlight ----------
	LCD_turnON_backlight;			//   GPIOB->BSRR = Bit(PIN_bk);
	
  /* Display On */
	LCD_WriteReg(0x29);				// Display ON (0x29)
}

/**
  * @brief  Disable the Display.
  * @param  None
  * @retval None
  */
void LCD_DisplayOff(void)
{
  /* Display Off */
	LCD_WriteReg(0x28);				// Display OFF (0x28)
	//---- turn off backlight ----------
	LCD_turnOFF_backlight;		//   GPIOB->BSRR = 0x01<<16;	
}

//###########################################################
//
//     Hardware Abstract Layer
//
//###########################################################
/**
  * @brief  Sets a display window by width (w) and height (h)
  *
*/
#define LCD_DisplayWindow_WnH(x, y, w, h)  LCD_OpenWin(x, y, x+w-1, y+h-1)

/**
  * @brief  Sets the LCD Text and Background colors.
  * @param  _TextColor: specifies the Text Color.
  * @param  _BackColor: specifies the Background Color.
  * @retval None
  */
void LCD_SetColors(uint16_t _TextColor, uint16_t _BackColor)
{
  DrawProp.TextColor = _TextColor;
  DrawProp.BackColor = _BackColor;
}

/**
  * @brief  Gets the LCD Text and Background colors.
  * @param  _TextColor: pointer to the variable that will contain the Text
            Color.
  * @param  _BackColor: pointer to the variable that will contain the Background
            Color.
  * @retval None
  */
void LCD_GetColors(uint16_t *pTextColor, uint16_t *pBackColor)
{
  *pTextColor = DrawProp.TextColor; 
	*pBackColor = DrawProp.BackColor;
}

uint16_t	LCD_GetTextColor(void)
{
		return DrawProp.TextColor;
}
uint16_t	LCD_GetBackColor(void)
{
		return DrawProp.BackColor;
}
/**
  * @brief  Sets the Text color.
  * @param  Color: specifies the Text color code RGB(5-6-5).
  * @retval None
  */
void LCD_SetTextColor(__IO uint16_t Color)
{
  DrawProp.TextColor = Color;
}


/**
  * @brief  Sets the Background color.
  * @param  Color: specifies the Background color code RGB(5-6-5).
  * @retval None
  */
void LCD_SetBackColor(__IO uint16_t Color)
{
  DrawProp.BackColor = Color;
}

/**
  * @brief  Sets the Text Font.
  * @param  fonts: specifies the font to be used.
  * @retval None
  */
void LCD_SetFont(sFONT *pfonts)
{
	DrawProp.pFont	= pfonts;	// pLCD_Currentfonts = pfonts;
	//----- modified by Shir-Kuan Lin
	wf = pfonts->Width; 	// font_width
	hf = pfonts->Height; // font_height
	bytes_wf = (wf+7) /8;	// number of bytes per width line in a font
	bytes_f = hf * bytes_wf;	// number of bytes per font (see Font12_Table or Font20_Table)
}

/**
  * @brief  Gets the Text Font.
  * @param  None.
  * @retval the used font.
  */
sFONT *LCD_GetFont(void)
{
  return pLCD_Currentfonts;
}

/**
  * @brief  Clears the selected line.
  * @param  Line: the Line to be cleared.
  *   This parameter can be one of the following values:
  *     @arg Linex: where x can be 0..n
  * @retval None
  */
void LCD_ClearStringLine(uint16_t LineNr)
{
  uint16_t bColor;
  uint16_t x, y;
	if (((LineNr+1) * pLCD_Currentfonts->Height) > (LCD_PIXEL_HEIGHT)){
		return;
	}
	LineNr = LineNr * pLCD_Currentfonts->Height;
	bColor = DrawProp.BackColor;
	
	LCD_OpenWin(0, LineNr, LCD_PIXEL_WIDTH, LineNr+hf-1);
  for(y = 0; y < hf; y++)		
  {
		for(x = 0; x <= LCD_PIXEL_WIDTH; x++)
		{
			LCD_WriteData(bColor);
		}
	}
}

/********************************************************************************************************
*  Function: LCD_DisplayChar				                                                           
*  Object: Display an ASCII character
*  Input: start point, end point, ASCII code, text color, background color
*  Output: none                                  
*  brief: none
********************************************************************************************************/
static void LCD_DisplayChar(uint16_t Xpos, uint16_t Ypos,  uint8_t Ascii, uint16_t fColor, uint16_t bColor)
{
	// wf =: font_width,
	// hf =:  font_height
// bytes_wf = number of bytes per width line in a font
// bytes_f  = number of bytes per font

	int	iw;
  uint32_t byteNr, jw;
	uint16_t  hp_limit;
	const uint8_t *c;
	
	if((Ascii > '~') || (Ascii < ' ')) Ascii= ' ';
		
	iw = Ascii - 32;  // make ' ' (space) at offset of 0
  c = &pLCD_Currentfonts->table[iw * bytes_f];	
	
	LCD_OpenWin(Xpos, Ypos, Xpos+wf-1, Ypos+hf-1);
	byteNr =0;
	hp_limit = Ypos + hf;
	
  for(; Ypos < hp_limit; Ypos++)		// font height 
  {
		uint8_t as;

		as = c[byteNr++];
		iw = 0;
    for(jw = 0; jw < wf; jw++)	//  loop for the font width (wf)
    {
				if(as & (0x80))
				{
					LCD_WriteData(fColor);
				}
				else
				{
					LCD_WriteData(bColor);
				}
				as <<= 1;
				iw++;
				if (iw == 8){							// reach 1 byte = 8 bits
					as = c[byteNr++];
					iw = 0;
				}
    }
	}
}

/**
  * @brief  Displays characters between a line segment on the LCD.
  * @param  Xpos: X position (in pixel)
  * @param  Ypos: Y position (in pixel)   
  * @param  Text: Pointer to string to display on LCD
  * @retval None
  */
static void LCD_DisplayStringSegment(uint16_t Xpos, uint16_t Ypos, char *Text) //, uint16_t font_width)
{
	uint16_t fColor, bColor;
	
	// wf =: font_width,
	// hf =:  font_height
// 	font_width = pLCD_Currentfonts->Width  // wf = font_width
	fColor = DrawProp.TextColor;
	bColor = DrawProp.BackColor;
  while ((*Text != 0) & (Xpos  <= (LCD_PIXEL_WIDTH - (wf-1))) )
  {
    /* Display one character on LCD */
    LCD_DisplayChar(Xpos, Ypos, (uint8_t)*Text, fColor, bColor);
    /* Decrement the column position by 16 */
    Xpos += wf;
    /* Point on the next character */
    Text++;
  }
}

/**
  * @brief  Displays characters on the LCD.
  * @param  Xpos: X position (in pixel)
  * @param  Ypos: Y position (in pixel)   
  * @param  Text: Pointer to string to display on LCD
  * @param  Mode: Display mode
  *          This parameter can be one of the following values:
  *            @arg  CENTER_MODE
  *            @arg  RIGHT_MODE
  *            @arg  LEFT_MODE   
  * @retval None
  */
void LCD_DisplayStringAt(uint16_t Xpos, uint16_t Ypos, char *Text, Line_ModeTypdef Mode)
{
  uint32_t leftcolumn=0;
  uint32_t size = 0, xBlank=0; 
  char  *ptr = Text;
  
	// wf =: font_width,
	// hf =:  font_height
// 	font_width = pLCD_Currentfonts->Width  // wf = font_width
	if (Mode == LEFT_MODE){
				leftcolumn = Xpos;				
	} else
	{
 /* Get the text size */
		while (*ptr++) size ++ ;
  
  /* Characters number per line */
		xBlank = ((LCD_PIXEL_WIDTH+1) /wf);		// maximum number of characters per line
		if (size > xBlank){
			xBlank = 0;
		}	else {		
			xBlank = (xBlank - size)* wf;			
		}
		xBlank += (LCD_PIXEL_WIDTH+1) %wf;		// width of blank space of this string line
	}
		switch (Mode)
		{
		case LEFT_MODE:
			{
				break;
			}
		case CENTER_MODE:
			{
				leftcolumn = Xpos + xBlank/ 2;
				break;
			}
		case RIGHT_MODE:
			{
				// if leftcolumn = xBlank, the right border of the string is at the right side of the screen
				leftcolumn = LCD_PIXEL_WIDTH - Xpos;
				if (leftcolumn > xBlank){
					leftcolumn = 0;
				} else {
					leftcolumn = xBlank - leftcolumn;
				}
				break;
			}
		}
	
  /* Send the string character by character on LCD */
	LCD_DisplayStringSegment((uint16_t) leftcolumn, Ypos, Text);
}


/**
  * @brief  Displays the string on the required row and column
  * @param ount: specifies the delay time length (time base 10 ms).
  * @retval : None
  */
void LCD_DisplayStringLineCol(uint8_t LineNr, uint16_t ColNr, char *ptr)
{
  uint16_t Xpos, Ypos ;
	
	// wf =: font_width,
	// hf =:  font_height
	Xpos = ColNr * wf;
	Ypos = LineNr * hf;
	
	LCD_DisplayStringSegment(Xpos, Ypos, ptr);
}
/**
  * @brief  Draws a pixel on LCD.
  * @param  Xpos: X position 
  * @param  Ypos: Y position
  * @param  Color: Pixel color in RGB mode (5-6-5)  
  * @retval None
  */
void LCD_DrawPixel(uint16_t Xpos, uint16_t Ypos, uint16_t Color)
{
		LCD_OpenWin(Xpos, Ypos, Xpos, Ypos);
	  LCD_WriteData(Color);
}

//------------------------------------------------------------------------------
static inline void swap_u16(uint16_t* a, uint16_t* b)
{
    uint16_t t = *a;
    *a = *b;
    *b = t;
}

//------------------------------------------------------------------------------
static inline void swap_i16(int16_t *a, int16_t *b)
{
    int16_t t = *a;
    *a = *b;
    *b = t;
}

#define LCD_DrawHLine2P(x0, y0, x1, y1) 	LCD_DrawHV2P(1, x0, y0, x1, y1)
#define LCD_DrawVLine2P(x0, y0, x1, y1) 	LCD_DrawHV2P(0, x0, y0, x1, y1)
//----------------------------------------------------
void LCD_DrawHV2P(uint8_t HLine, uint16_t x0, uint16_t y0, uint16_t x1, uint16_t y1)
{
		uint16_t length_m_1;
    // Horizontal Line	(y0 = y1)
		if (HLine)	{
			y1 = y0;	// make sure	
			if (x0 > x1) 	swap_u16(&x0, &x1);		// Now x0 <= x1
			length_m_1 = x1 - x0;
		} else {
			//  Vertical Line	(x0 = x1); HLine = 0: 
			x1 = x0;	// make sure	
			if (y0 > y1) 	swap_u16(&y0, &y1);		// Now y0 <= y1
			length_m_1 = y1 - y0;
		}
		LCD_OpenWin(x0, y0, x1, y1);	

	uint16_t Tcolor=DrawProp.TextColor;
	for(uint16_t i = 0; i <= length_m_1; i++)
			 LCD_WriteData(Tcolor);
}

/**
  * @brief  Draws a horizontal line on LCD.
  * @param  Xpos: X position 
  * @param  Ypos: Y position
  * @param  Length: line length
  * @param  RGB_Code: Pixel color in RGB mode (5-6-5)  
  * @retval None
  */
void LCD_DrawHLine(uint16_t Xpos, uint16_t y0, uint16_t Length)
{
//	uint16_t color=DrawProp.TextColor;
	uint16_t x1;

	if (y0 > LCD_PIXEL_HEIGHT) return;
	x1 = Xpos + Length - 1;
	if ( x1 > LCD_PIXEL_WIDTH)	x1 = LCD_PIXEL_WIDTH;
	
	LCD_DrawHLine2P( Xpos, y0, x1, y0); 	// y0 = y1
}

/**
  * @brief  Draws a vertical line on LCD.
  * @param  Xcen: X position 
  * @param  Ypos: Y position
  * @param  Length: line length
  * @param  RGB_Code: Pixel color in RGB mode (5-6-5)  
  * @retval None
  */
void LCD_DrawVLine(uint16_t x0, uint16_t Ypos, uint16_t Length)
{
//	uint16_t color=DrawProp.TextColor;
	uint16_t y1;

	if (x0 > LCD_PIXEL_WIDTH) return;
	y1 = Ypos + Length - 1;
	if ( y1 > LCD_PIXEL_HEIGHT)	y1 = LCD_PIXEL_HEIGHT;

	LCD_DrawVLine2P( x0, Ypos, x0, y1);	// x0 = x1
}

/**
  * @brief  Draws a simple cross on LCD.
  * @param  Xcen: X center position 
  * @param  Ypos: Y center position
  * @param  Length: line length
  * @param  RGB_Code: Pixel color in RGB mode (5-6-5)  
  * @retval None
  */
void LCD_DrawSimpleCross(uint16_t Xpos, uint16_t Ypos, uint16_t Length)
{
	uint16_t hL;
	if ((Length & Bit(0)) == 0)		//even value
			Length +=1;
	
	hL = Length/2;
	LCD_DrawVLine(Xpos, Ypos-hL, Length);
	LCD_DrawHLine(Xpos-hL, Ypos, Length);
}

//--------------------------------------------------
static void LCD_DrawRect2P(uint16_t Xul, uint16_t Yul, uint16_t Xdr, uint16_t Ydr)
{
	//===  horizontal lines
	LCD_DrawHLine2P( Xul, Yul, Xdr, Yul); 	// y0 = y1
	LCD_DrawHLine2P( Xul, Ydr, Xdr, Ydr); 	// y0 = y1

	//===  vertical lines
	LCD_DrawVLine2P( Xul, Yul, Xul, Ydr); 	// x0 = x1
	LCD_DrawVLine2P( Xdr, Yul, Xdr, Ydr); 	// x0 = x1
}
/**
  * @brief  Displays a rectangle border.
  * @param  Xpos: specifies the X position.
  * @param  Ypos: specifies the Y position.
  * @param  Height: display rectangle height.
  * @param  Width: display rectangle width.
  * @retval None
  */
void LCD_DrawRect(uint16_t Xpos, uint16_t Ypos, uint16_t Width, uint16_t Height)
{
	uint16_t x1, y1;
	//===  horizontal lines
  LCD_DrawHLine(Xpos, Ypos, Width);
	x1 = (Xpos + Width - 1);
	y1 = (Ypos + Height- 1);
	if (x1 > LCD_PIXEL_WIDTH) x1 = LCD_PIXEL_WIDTH;
	if ( y1 > LCD_PIXEL_HEIGHT)	y1 = LCD_PIXEL_HEIGHT;
	
	LCD_DrawRect2P(Xpos, Ypos, x1, y1);
}
//--------------------------------------------------
void LCD_DrawRect_by_2Points(Point *pA)
{
	uint16_t x0,y0, x1, y1;
	x0 = pA[0].X;
	y0 = pA[0].Y;
	x1 = pA[1].X;
	y1 = pA[1].Y;

	LCD_DrawRect2P(x0, y0, x1, y1);	
}

/**
  * @brief  Displays a circle border.
  * @param  Xcen: specifies the X position of the center.
  * @param  Ycen: specifies the Y position of the center.
  * @param  Radius
  * @retval None
  */
void LCD_DrawCircle(uint16_t Xcen, uint16_t Ycen, uint16_t Radius)
{
	uint16_t color=DrawProp.TextColor;

  int32_t  next;/* Decision Variable */
  uint16_t  Ri;/* increasing from 0 to Radius */
  uint16_t  Rd;/* decreasing from Radius to 0 */

  next = 3 - (Radius << 1);
  Ri = 0;
  Rd = Radius;

  while (Ri <= Rd)
  {
		int32_t x0, y0;

		x0 = Xcen -	Ri;
		y0 = Ycen -	Rd;
		if(x0<0) x0=0;
		if(y0<0) y0=0;
				LCD_DrawPixel((uint16_t)x0, (uint16_t)y0, color);                          
				LCD_DrawPixel((uint16_t)x0, Ycen+Rd, color);                  
				LCD_DrawPixel(Xcen+Ri, (uint16_t)y0, color);             
				LCD_DrawPixel(Xcen+Ri, Ycen+Rd, color);             
		x0 = Xcen -	Rd;
		y0 = Ycen -	Ri;
		if(x0<0) x0=0;
		if(y0<0) y0=0;
				LCD_DrawPixel((uint16_t)x0, (uint16_t)y0, color);              	         
				LCD_DrawPixel((uint16_t)x0, Ycen+Ri, color);             
				LCD_DrawPixel(Xcen+Rd, (uint16_t)y0, color);                      
				LCD_DrawPixel(Xcen+Rd, Ycen+Ri, color);                          

    if (next < 0)
    {
      next += (Ri << 2) + 6;
    }
    else
    {
      next += ((Ri - Rd) << 2) + 10;
      Rd--;
    }
    Ri++;
  }
}

//--------------------------------------------------------------------------
static void LCD_FillRect2P(uint16_t Xpos, uint16_t Ypos, uint16_t Xend, uint16_t Yend)
{
	uint16_t color=DrawProp.TextColor;
	
		LCD_OpenWin(Xpos, Ypos, Xend, Yend);
		for(uint16_t j = Ypos; j <= Yend; j++)
			 for(uint16_t i = Xpos; i <= Xend; i++)
					 LCD_WriteData(color);
}
/**
  * @brief  Displays a full rectangle with TestColor.
  * @param  Xpos: specifies the X position.
  * @param  Ypos: specifies the Y position.
  * @param  Height: rectangle height.
  * @param  Width: rectangle width.
  * @retval None
  */
void LCD_FillRect(uint16_t Xpos, uint16_t Ypos, uint16_t Width, uint16_t Height)
{
	uint16_t color=DrawProp.TextColor;
	uint16_t Xend = Xpos+ Width-1, Yend = Ypos + Height-1;
	
		LCD_OpenWin(Xpos, Ypos, Xend, Yend);
		for(uint16_t j = Ypos; j <= Yend; j++)
			 for(uint16_t i = Xpos; i <= Xend; i++)
					 LCD_WriteData(color);
}

/**
  * @brief  Displays a full circle.
  * @param  Xpos: specifies the X position.
  * @param  Ypos: specifies the Y position.
  * @param  Radius
  * @retval None
  */
void LCD_FillCircle(uint16_t Xcen, uint16_t Ycen, uint16_t Radius)
{
//	uint16_t color=DrawProp.TextColor;
  int32_t  next;/* Decision Variable */
  uint16_t  Ri;/* increasing from 0 to Radius */
  uint16_t  Rd;/* decreasing from Radius to 0 */

  next = 3 - (Radius << 1);
  Ri = 0;
  Rd = Radius;

  while (Ri <= Rd)
  {
		int32_t X0, Y0;

		X0 = Xcen - Ri;
		Y0 = Ycen -	Rd;
		if(X0<0) X0=0;
		if(Y0<0) Y0=0;
		LCD_DrawVLine((uint16_t)X0, (uint16_t)Y0, (uint16_t)(Ycen + Rd - Y0));				// vertical line in landscape  view
		X0 = Xcen + Ri;
		LCD_DrawVLine((uint16_t)X0, (uint16_t)Y0, (uint16_t)(Ycen + Rd - Y0));				// vertical line in landscape  view
		X0 = Xcen - Rd;
		Y0 = Ycen -	Ri;
		if(X0<0) X0=0;
		if(Y0<0) Y0=0;
		LCD_DrawHLine((uint16_t)X0, (uint16_t)Y0, (uint16_t)(Xcen + Rd - X0));				// horizontal line in landscape  view
		Y0 = Ycen +	Ri;
		LCD_DrawHLine((uint16_t)X0, (uint16_t)Y0, (uint16_t)(Xcen + Rd - X0));				// horizontal line in landscape  view
    if (next < 0)
    {
      next += (Ri << 2) + 6;
    }
    else
    {
      next += ((Ri - Rd) << 2) + 10;
      Rd--;
    }
    Ri++;
  }
}

//------------------------------------------------
//-----------------------------------------------------------------------			
//  positive integers: dy>0, dx>=dy>0, m = dx/2, h= y-y1 >= 0, i, 0<=r<dx.
//
//	a.  It is known that  (i*dy  + m) = h*dx + r, 0<=r<dx, h = integer[(i*dy  + m)/dx],  r = (i*dy  + m)%dx  			
//			 Note, h*dx + (v*dy + k) = (i*dy  + m), where v*dy+k = r, or v = integer(r/dy), k = r % dy < dy
//			 ==> h*dx - m = (i-v-1)*dy + (dy-k) = q * dy + z, where q=(i-v-1) = integer((h*dx - m)/dy ), z = dy-k = (h*dx - m)%dy < dy
//
//			==> h*dx - m = q*dy + z, where z = (h*dx-m) % dy < dy, q =(integer((h*dx - m)/dy )
//	Question: Find minimum w such that s = ((q+w)*dy + m)/dx >= h ==> (y1 + s) >= y1 + h = y
//        Note, when dx > dy, there are many (q+w) for a single h.
//
//	Since 1. q*dy = (h*dx - m) -z < (h*dx - m) ==> q*dy + m < h*dx
//				2. (q+1)*dy = q*dy + dy > q*dy + z = h*dx - m	since dy > z
//				When z = 0, q*dy = h*dx - m ==> w = 0; and if w >=1, then q*dy+w*dy > h*dx - m.
//				When z > 0, q*dy+w*dy = h*dx - m + w*dy -z > h*dx-m for any w >= 1, since (w*dy-z)>= (dy-z)>0; 
//	SOL. for a. ==> w_min = 0 when z = 0, or w_min = 1, when z >0.
//				Code: (q+w_min) = min_x_for_y(h, dy, dx): q=integer((h*dx - m)/dy ), z = ((h*dx - m)%dy
//			
//	b.  It is known that  q*dy + z = (h+1)*dx - m, 0<=z<dy, z = ((h+1)*dx- m) % dy		
//	    Find MAXIMUM w such that s = (q+w)*dy /dx + m < (h+1) ==> (y1 + s) < y1+(h+1) = y+1
//
//	Since q*dy <= q*dy + z = (h+1)*dx -m,  z = ((h+1)*dx- m) % dy		
//				When z = 0, (q-1)*dy = q*dy - dy < (h+1) * dx -m, since q*dy = (h+1)*dx -m ==> z = 0,  w_max = -1  
//				When z > 0, q*dy < q*dy + z = (h+1) * dx -m ==> z > 0,  w_max = 0  
//	SOL. for b. ==> w_max = -1 when z = 0, or w_max = 0, when z >0.
//  ==> Code: max_x_for_yPLUS1(h, dy, dx)
//	
//	For instance, dy = 4, dx = 14 in the following		
//        min   Max min  Max min     Max
// 	    0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17
//	h=0	o  x  x  X
//	h=1	(q=3, z=1)  x  x  X 
//	h=2	(q=7, z=0)           x  x  x  X
//	h=3	(q=10, z=1)                      x  x  X
//	h=4	(q=14, z=0)                               o  x  x  X
//-----------------------------------------------------------------------
//		x shift left by (dx/dy)/2 ==> new_i = i+(dx/dy)/2, i.e., increase i by (dx/dy)/2
//			(d/n)/2 = 1
// 	    0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17
//	h=0	o  x  X
//	h=1	         x  x  X 
//	h=2	(q=7, z=0)        x  x  x  X
//	h=3	(q=10, z=1)                   x  x  X
//	h=4	(q=14, z=0)                            x  o  x  X
//-----------------------------------------------------------------------
//------------ For dy < 0, dx>=|dy| > 0, m =-dx/2 < 0 and h = y-y1 <= 0 in the following  -------------			
//	c.  It is known that   h*dx - m = q*dy + z, 0>=z>dy, z = (h*dx- m) % dy
//			 			
//	    Find minimum w such that s=((q+w)*dy +m) /dx  <= h ==> y1 + s <= y1 + h = y
//							q*(-dy) + (-z) = (-h)*dx - (-m), 0<=(-z)<(-dy), (-z) = ((-h)*dx- (-m)) % (-dy)
//	Since (q+1)*(dy) = q*(dy) + (dy) < q*(dy) + (z) = (h)*dx - (m)	since dy < z
//				When z = 0, q*dy = h*dx - m ; and q*dy+(+1)*dy < h*dx - m, since dy < 0.
//				When z < 0, q*dy+(+1)*dy = h*dx + (dy) -m -z < h*dx-m, since dy-z < 0.
//	SOL. for c. ==> w_min = 0 when z = 0, or w_min = 1, when (-z) >0.
//  ==> Code: min_x_for_y(-h, -dy, dx)
//			
//	d.  It is known that  q*dy + z = (h+(-1))*dx - m, 0>=z>dy, z = ((h-1)*dx- m) % dy			
//	    Find MAXIMUM w such that ((q+w)*dy - m) /dx > h-1 
//					==> Find MAXIMUM w such that ((q+w)*(-dy) - (-m)) /dx < (-h) + 1
//	Since (q+1)*(-dy) = q*(-dy) + (-dy) > q*dy + z = (-h)*dx	
//	SOL. for d. ==> w_min = -1 when z = 0, or w_min = 0, when z >0.
//  ==> Code: max_x_for_yPLUS1(-h, -dy, dx)
//
//	For instance, dy = -4, dx = 14 in the following		
//        min   Max   min  Max min     Max
// 	       0  1   2   3  4  5  6  7  8  9 10  11 12 13 14 15 16 17
//	h=-4	(q=14, z=0)                                   o  x  x  X
//	h=-3	(q=10, z=-2)                         x  x  X
//	h=-2	(q=7, z=0)              x  x  x  X
//	h=-1	(q=3, z=-2)    x  x  X 
//	h=-0	 o  x   x   X
//-----------------------------------------------------------------------
//		x shift left by (dx/|dy|)/2 ==> new_i = i+(dx/|dy|)/2, i.e., increase i by (dx/|dy|)/2
//			(dx/|dy|)/2 = 1
//        min   Max   min  Max min     Max
// 	   0   1  2  3     4  5  6  7  8  9  10 11 12 13 14 15 16 17
//	h=-4	(q=14, r=0)                              x  o  x  X
//	h=-3	(q=10, z=-2)                   x  x  X
//	h=-2	(q=7, r=0)         x  x  x  X
//	h=-1	        x    x   X 
//	h=-0	 o  X
//-----------------------------------------------------------------------			
static uint16_t min_x_for_y(int16_t h, int16_t dy, uint16_t dx)
	// h > 0, dy > 0, dx > 0
{
	uint16_t z, q;
	int32_t y_t;
	
	//>>--- just for debug
	if (dy <= 0) while(1);
	if (dy > dx )	while(1);
	if (h < 0 ) while(1);
	//<<<-----------------
	y_t = h * dx - dx/2;
	q = y_t / dy;	// q = (h*dx- dx/2) / dy
	z = y_t % dy;	// z = (h*dx- dx/2) % dy
	if ( z == 0) return q;		// w_min = 0
	else	return q+1;					// w_min = 1
}
static uint16_t max_x_for_y(int16_t h, int16_t dy, uint16_t dx)
// h > 0, dy > 0, dx > 0
{
	uint16_t z, q;
	int32_t y_t;
	
	//>>--- just for debug
	if (dy <= 0) while(1);
	if (dy > dx ) while(1);
	if (h < 0 ) while(1);
	//<<<-----------------
	y_t = (h+1) * dx - dx/2;
	q = y_t / dy;	// q = ((h+1)*dx- dx/2) / dy
	z = y_t % dy; // r = ((h+1)*dx- dx/2) % dy
	if ( z == 0) return q-1;		// w_max = -1
	else	return q;							// w_max = 0;
}

//----------------------------------------------------------------------
static inline void DecomposeImStr(uint16_t* X0, uint16_t* Y0, uint16_t* width, uint16_t* height, uint16_t* X_most, uint16_t* Y_most, sImageBuf *pImageSt) 
{
			*X0 = pImageSt->topLeft.X;
			*Y0 = pImageSt->topLeft.Y;
			*width = pImageSt->width;
			*height = pImageSt->height;
			*X_most = *X0 + *width - 1;
			*Y_most = *Y0 + *height - 1;
}
////==============================================================
//------------------------------------------------------------------------------
static inline uint8_t adjust_x1_less_x2_inRange(uint16_t* p1, uint16_t* p2, uint16_t P0, uint16_t P_most)
//	Output value: 0=no more doing; 1 = OK and continue
//------------------------------------------------------------------------------
{
//	uint16_t width, height;
		if ((*p1 > P_most) || (*p2 < P0) ) return 0;	// not in the range of (X0, Y0) ~ (X_most, Y_most)
		if (*p1 < P0) *p1 = P0;			// initial point
		if (*p2 > P_most) *p2 = P_most;	// end point
	return 1;
}

//-----------------------------------------------------------
static inline void adjust_y1_y2_inRange(int16_t dy, int16_t *h1, int16_t *h2, uint16_t *y1, uint16_t *y2,
						uint16_t org_y1, uint16_t Y0, uint16_t Y_most)
{
			if (dy > 0) {
			// org_y1 = y1 < y2
				if ( *y1 < Y0 ){
					*y1 = Y0;					// org_y1 < Y0
					*h1 = Y0 - org_y1;		// h1 > 0
				}
				if (*y2 > Y_most){		// y2 > Y_most 
					*y2 = Y_most;		// y2 > org_y1 ==> Y_most >= org_y1
					*h2 = Y_most - org_y1;		// h2 >= 0	
					// h2 = 0, when org_y1 = Y_most
				}
			} else if (dy < 0) {
			// org_y1=y1 > y2
				if (*y1 > Y_most){		
					*y1 = Y_most;			// org_y1 > Y_most
					*h1 = org_y1 - Y_most;	// - (Y_most - org_y1) > 0
				}
				if (*y2 < Y0){		// y2 < Y0 
					*y2 = Y0;			// y2 > org_y1 ==> Y0 <= org_y1
					*h2 = org_y1 - Y0; // - (Y0 - org_y1) >= 0
					// h2 = 0, when org_y1 = Y0
				}
			}	
}

	volatile int16_t	gy1, gy2, gx1, gx2;
	volatile int32_t gy, gL, gR, gdx, gdy;

//=============================================================================================
static inline void PlusMinus_DrawLine_RGBBuffer(uint8_t swapXY, uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2,
		 sImageBuf *pImageSt) 
{
			uint16_t X0, Y0, width, height;
			uint16_t *pBuffer;
			uint16_t X_most, Y_most;

			DecomposeImStr(&X0, &Y0, &width, &height, &X_most, &Y_most, pImageSt);
			pBuffer = pImageSt->data;
	// Now x1<=x2;
	if (swapXY) {
		swap_u16(&X0, &Y0);
		swap_u16(&X_most, &Y_most);
	}
	if ((y1 < Y0) && (y2 < Y0)) 				return;
	if ((y1 > Y_most) && (y2 > Y_most)) return; 	// not in the range of (X0, Y0) ~ (X_most, Y_most)
	
	uint16_t org_x1 = x1;
	uint16_t org_y1 = y1;
	uint16_t org_x2 = x2;
	if (! adjust_x1_less_x2_inRange(&x1, &x2, X0, X_most) ) return;

	uint16_t	dx	= org_x2 - org_x1;		// note: dx > 0 definitely
	int16_t		dy = y2 - y1;
	int16_t 	h_dx = (dy >= 0) ? dx/2: -dx/2;
		if ((x1 > org_x1))	y1 = org_y1 + ((x1 - org_x1) * dy + h_dx) / dx;	
		if ((x2 < org_x2))	y2 = org_y1 + ((x2 - org_x1) * dy + h_dx) / dx;	
			
		uint16_t	n = (dy >= 0) ? dy : -dy;
		int16_t	h1=-1, h2=-1;
		adjust_y1_y2_inRange(dy, &h1, &h2, &y1, &y2, org_y1, Y0, Y_most);
			if (h1 > 0) 	x1 = org_x1 + min_x_for_y(h1, n, dx);
			if (h2 >= 0)	x2 = org_x1 + max_x_for_y(h2, n, dx);
			// Now, x1 >= X0, y1 >= Y0; x2 <= X_most, y2 <= Y_most
	//----------------------------------
		
		uint16_t y_last;
		int16_t Dn_row = (dy >=0) ? (int16_t)width : (int16_t)-width;
		int16_t Rt_col = (dy >=0) ? +1 : -1;
		//--- for (x, y) =  (x1, y1)
			uint16_t x_m_X0 = (!swapXY) ? (x1 - X0) : (y1 -Y0);	// for x = x1
			uint32_t y_m_Y0_width = (!swapXY) ? (y1 -Y0) : (x1 - X0);	// for y = y1
			y_m_Y0_width *= width;

	//----------------------------------
	uint16_t r = dx / 2, abs_dy = ABS(dy);

	int16_t inc_y = (dy >= 0) ? 1 : -1;	// True =1, False = -1
	int32_t y_int = y1;
	if (x1 > org_x1) {
		r = r + abs_dy * (x1-org_x1) - ABS(y1-org_y1) * dx;
	}

//	uint16_t Tcolor=DrawProp.TextColor;
	for (uint16_t x = x1; x <= x2; x++) {
 		uint32_t offset; 

			offset = y_m_Y0_width + x_m_X0;		
			pBuffer[offset] = DrawProp.TextColor;
////		if (swapXY==0) {	// not swaped x, y 
//// 					LCD_DrawPixel(x, y_int, RED);
////     } else {					// has swapped x, y 
//// 					LCD_DrawPixel(y_int, x, RED);
////     }
			y_last = y_int;
//-------------------------
// 			Iteration Mehod 	
//---------------------------------------------------------------------------------			
			r += abs_dy;
			if (r >= dx){
				r -= dx;
				y_int += inc_y;
			}
	//-------- for calculating variable "offset"
			if (swapXY==0){
				if (y_int != y_last) y_m_Y0_width += Dn_row;	// +width for dy >0; -+width for dy <0
				x_m_X0 ++;		// x_m_X0 =	x+1 - X0
			} else {		// for swapXY = 1
				if (y_int != y_last) x_m_X0 += Rt_col;		// +1 for dy >0; -1 for dy <0
				y_m_Y0_width += width; // y_m_Y0_width = (x+1 - X0) * width	
			}
  }
		//----------------------------------

}

//==============================================================
static inline void Integer_DrawLine_RGBBuffer(uint8_t swapXY, uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2,
		 sImageBuf *pImageSt) 
{
			uint16_t X0, Y0, width, height;
			uint16_t *pBuffer;
			uint16_t X_most, Y_most;

			DecomposeImStr(&X0, &Y0, &width, &height, &X_most, &Y_most, pImageSt);
			pBuffer = pImageSt->data;
	// Now x1<=x2;
	if (swapXY) {
		swap_u16(&X0, &Y0);
		swap_u16(&X_most, &Y_most);
	}
	if ((y1 < Y0) && (y2 < Y0)) 				return;
	if ((y1 > Y_most) && (y2 > Y_most)) return; 	// not in the range of (X0, Y0) ~ (X_most, Y_most)
	
	uint16_t org_x1 = x1;
	uint16_t org_y1 = y1;
	uint16_t org_x2 = x2;
	if (! adjust_x1_less_x2_inRange(&x1, &x2, X0, X_most) ) return;

	uint16_t	dx	= org_x2 - org_x1;		// note: dx > 0 definitely
	int16_t		dy = y2 - y1;
	int16_t 	h_dx = (dy >= 0) ? dx/2: -dx/2;
		if ((x1 > org_x1))	y1 = org_y1 + ((x1 - org_x1) * dy + h_dx) / dx;	
		if ((x2 < org_x2))	y2 = org_y1 + ((x2 - org_x1) * dy + h_dx) / dx;	
			
		uint16_t	n = (dy >= 0) ? dy : -dy;
		int16_t	h1=-1, h2=-1;
		adjust_y1_y2_inRange(dy, &h1, &h2, &y1, &y2, org_y1, Y0, Y_most);
			if (h1 > 0) x1 = org_x1 + min_x_for_y(h1, n, dx);
			if (h2 >= 0)	x2 = org_x1 + max_x_for_y(h2, n, dx);
			// Now, x1 >= X0, y1 >= Y0; x2 <= X_most, y2 <= Y_most
		h2 = org_y1 + ((x2 - org_x1) * dy + h_dx) / dx;	
		if (h2 != y2) while(1);
	//----------------------------------
		

		uint16_t y_last;
		int16_t Dn_row = (dy >=0) ? (int16_t)width : (int16_t)-width;
		int16_t Rt_col = (dy >=0) ? +1 : -1;
		//--- for (x, y) =  (x1, y1)
			uint16_t x_m_X0 = (!swapXY) ? (x1 - X0) : (y1 -Y0);	// for x = x1
			uint32_t y_m_Y0_width = (!swapXY) ? (y1 -Y0) : (x1 - X0);	// for y = y1
			y_m_Y0_width *= width;

		uint16_t y = y1;
		int16_t inc = (dy >= 0) ? dx/2: -dx/2;
		inc += (x1 - org_x1) * dy;
		// Note: inc = (x1 - org_x1) * dy + sgn(dy) * dy/2;
	for (uint16_t x = x1; x <= x2; x++) {
		uint32_t offset; 

//#define Debug_rigorousness
#ifdef Debug_rigorousness
 		if ((y < Y0) || (y > Y_most)) {	
			return;			// no more inside the range
		} else { //		if ((y >= Y0) && (y <=Y_most)) {	
#endif		
			offset = y_m_Y0_width + x_m_X0;		
			pBuffer[offset] = DrawProp.TextColor;
/*--------- Want to debug, then uncomment the following lines  -----*/
//#define Debug_Buffer
#ifdef Debug_Buffer
		uint16_t Tcolor = RED;
		uint32_t tmpx, tmpy;
			if (swapXY==0) {
				LCD_DrawPixel(x, y, Tcolor);
				tmpx = x-X0; 
				tmpy = (y-Y0)*width;
      } else 	{	// x <--> y;  X0 <--> Y0
				LCD_DrawPixel(y, x, Tcolor);
				tmpx = (y-Y0);
				tmpy = (x-X0) * width ;
			}
				if (tmpx != x_m_X0) while(1);
				if (tmpy != y_m_Y0_width) while(1);
			}
#endif
//<<<<<<<---------------------------------------------------------			
#ifdef Debug_rigorousness
		}		// END of if ((y >= Y0) && (y <=Y_most))
#endif
		y_last = y;
		inc += dy; 								// for x+1: ((x+1) - org_x1) * dy + sgn(dy) * dy/2
		y = org_y1 + (inc) / dx;	// for x+1: y = org_y1 + ((x+1-org_x1) * dy + dx/2) / dx;
	//-------- for calculating variable "offset"
		if (!swapXY){
			if (y != y_last) y_m_Y0_width += Dn_row;	// +width for dy >0; -+width for dy <0
			x_m_X0 ++;		// x_m_X0 =	x+1 - X0
		} else {		// for swapXY = 1
			if (y != y_last) x_m_X0 += Rt_col;		// +1 for dy >0; -1 for dy <0
			y_m_Y0_width += width; // y_m_Y0_width = (x+1 - X0) * width	
		}
	}

}

//--------------------------------------------------------------------------
static void	draw_HVline_inBuffer(uint8_t HLine, uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2, sImageBuf *pImageSt) 
{
			uint16_t X0, Y0, width, height;
			uint16_t *pBuffer;
			uint16_t X_most, Y_most;

			DecomposeImStr(&X0, &Y0, &width, &height, &X_most, &Y_most, pImageSt);
			pBuffer = pImageSt->data;

		uint16_t length_m_1;
    // Horizontal Line	(y0 = y1)
		if (HLine)	{
			y2 = y1;	// make sure	
			if ((y1 < Y0) || (y1 > Y_most)) return; // not in the range of (X0, Y0) ~ (X_most, Y_most)
			if (x1 > x2) 	swap_u16(&x1, &x2);		// Now x1 <= x2
			if (!adjust_x1_less_x2_inRange(&x1, &x2, X0, X_most) ) return;
			length_m_1 = x2 - x1;
		} else {
			//  Vertical Line	(x0 = x1); HLine = 0: 
			x2 = x1;	// make sure	
			if ((x1 < X0) || (x1 > X_most)) return; // not in the range of (X0, Y0) ~ (X_most, Y_most)
			if (y1 > y2) 	swap_u16(&y1, &y2);		// Now y1 <= y2
			if (!adjust_x1_less_x2_inRange(&y1, &y2, Y0, Y_most) ) return;
			length_m_1 = y2 - y1;
		}
	
		uint16_t offset = (y1 -Y0) * width + (x1 - X0);	// initial value

		for (uint16_t i = 0; i <= length_m_1; i++){		// do only when X_most >= x2 >= x1 >= X0
			pBuffer[offset] = DrawProp.TextColor;
			if (HLine)	offset ++;		// for horizontal line
			else 	offset += width; // for vertical line	
		}
}		

//=======================================================================
void LCD_DrawLine_RGBbuffer(uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2, sImageBuf *pImageSt)
{
		if (pImageSt == 0) return;  // no image buffer

	uint16_t deltaX, deltaY;

	deltaX = (uint16_t)ABS(x2 - x1);        /* The difference between the x's */
  deltaY = (uint16_t)ABS(y2 - y1);        /* The difference between the y's */
	//>>>>--------------------------------------------------------
	
	if ((deltaY == 0) || (deltaX == 0)) {
			// Horizontal Line (y1 = y2)
		uint8_t HLine = 1;
			// Vertical Line	(x1 = x2)
		if (deltaX == 0) HLine = 0;
			draw_HVline_inBuffer(HLine, x1, y1, x2, y2, pImageSt);	// y1 = y2
			return;
	}

     // General:      (x1 != x2, otherwise 0 = deltaX = deltaY = dy = 0)
	uint8_t swapXY = deltaX < deltaY;
   if (swapXY) {		// if (ABS(y1 - y0) > ABS(x1 - x0))
			swap_u16(&x1, &y1);
			swap_u16(&x2, &y2);
    }	
		// now, ABS(x2 - x1) >= ABS(y2 - y1)
   if (x1 > x2) {
			swap_u16(&x1, &x2);
			swap_u16(&y1, &y2);
    }		// Now x0 <= x1

			PlusMinus_DrawLine_RGBBuffer(swapXY, x1, y1, x2, y2, pImageSt);		
//			Integer_DrawLine_RGBBuffer(swapXY, x1, y1, x2, y2, pImageSt);
}

/**
  * @brief  Displays an uni-line (between two points).
  * @param  x1: specifies the point 1 x position.
  * @param  y1: specifies the point 1 y position.
  * @param  x2: specifies the point 2 x position.
  * @param  y2: specifies the point 2 y position.
  * @retval None
  */
	//------------------------------------------------

#define Q16 16
static inline void Qformat_DrawLine(uint8_t swapXY, uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2)
//-----------------------------------------	
//		Line is shifted left by (dx/|dy|)/2 ==> new_i = i + (dx/|dy|)/2 and h[0]=1/2;
//			(new_i) * dy = i* dy + (dx/2) * (dy/|dy|) = i*dy + sgn(dy) * (dx/2)
//			define m =: sgn(dy) * (dx/2);		
//		==> Floating Point:	 i * dy + m = h[i] * dx, so that h[0] = m/dx = sgn(dy) * 1/2
//		==> Round off Method: h[i] = (i * dy )/dx +1/2 
//		==> Q format rule for negative: 
//				-5.1 ==>	(-51<<8)/10 >>8 = (-13056/10) >>8 = (-1305) >>8 = -6
//				-5.5 ==>	(-55<<8)/10 >>8 = (-14080/10) >>8 = (-1408) >>8 = -6
//				-5.9 ==> 	(-59<<8)/10 >>8 = (-15104/10) >>8 = (-1510) >>8 = -6
//
//	  ==> Q format:  g = (dy<<16)/dx;	M = 1/2 = 1 <<15;
//	                 h[i] = M + (i * g); 
//                   y[i] = y1 + h[i] >>16 = [(y1<<16 + M) + (i*g)] >>16; 
//-----------------------------------------	
//	For instance, dy = 4, dx = 14 in the following: (dx/dy)/2 = 1.75, dx/2 = 7		
// 	    0  1   2   3  4   5   6  7   8   9   10  11  12  13  14 
//	h=0	xo xo  xo  X																						(3*4/14= 0.85) ==> ((3+1.75)*(4))/14) = (3*(4) + 1.75*4) /14) = 19/14 = 1.35
//	h=1	  				 o  xo  xo Xo
//	h=2	           |<-|				   xo  xo  xo  X
//	h=3	                     						      o  xo  Xo  X      (13*4/14= 3.71) ==> ((13+1.75)*(4))/14) = (3*(4) + 1.75*4) /14) = 59/14 = 4.21
//	h=4	                             							       o   xo
//-----------------------------------------------------------------------
//	For instance, dy = -4, dx = 14 in the following	(dx/|dy|)/2 = 1; dx/2 = 7		
//        min   Max   min  Max min     Max
// 	   0   1   2   3   4   5   6   7   8   9  10  11  12  13  14
//	h=-4	                                                 o  xo  (-14*4/14= -4) 
//	h=-3	                                     o  xo  xo  X				(-13*4/14= -3.71)
//	h=-2	          |<-|           xo  xo  xo  X									(-10*4/14= -2.85) ==> (32768+ 10*(-18724)) >> 16 = -3
//	h=-1	          o  xo  xo  Xo 																(-4*4/14= -1) 
//	h=0	   xo  xo  X																							(-3*4/14= -0.85) ==> ((1<<15) + 3 * [(-4)<<16/dx] ) = (32768+ 3*(-18724)) >> 16 = -1
//-----------------------------------------------------------------------
//-----------------------------------------	
{
   uint16_t dx = x2 - x1;
   int16_t dy = y2 - y1;
    int32_t gradient = (dx == 0) ? 0 : ((int32_t)dy << Q16) / dx;
		// Note: dx >= ABS(dy), if dx==0, then dy == 0
		//--- dx!=0 && dy==0 ==> gradient = 0 a horizontal line (swapXY=0) or a vertical line (swapXY=1);

    int32_t y = y1 << Q16;	
						y += 1<< (Q16 -1);  // += 0.5 << 16 for Round off
		
		uint16_t Tcolor=DrawProp.TextColor;
    for (uint16_t x = x1; x <= x2; x++) {
			int32_t y_int = y >> Q16;		// (y) >> FP_SHIFT
      if (swapXY==0) {	// not swap x, y 
 					LCD_DrawPixel(x, y_int, Tcolor);
      } else {					// has swapped x, y 
 					LCD_DrawPixel(y_int, x, Tcolor);
      }
       y += gradient;
    }
	}

static inline void PlusMinus_DrawLine(uint8_t swapXY, uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2)
//-----------------------------------------	
//			y = y1 + ((x-x1) * dy + sgn(dy) * dx/2) / dx;	// from original y1 to y2
//                     -------------------------
//			When along y:   (x-x1) * dy  + z = (y-y1) * dx + my, where z = ((y-y1) * dx - m) % dy  
//     ==> i=(y-y1); inc[i] = i*dx + my = inc[i-1] + dx; x[i] = x1 + inc[i]/dy;
//-----------------------------------------	
//		==> Iteration for dy > 0: r[0] = dy/2 for dy >= dx; r[0] = -|dx|/2 for dy < dx.
//		==> Iteration:  inc_x = (dx > 0) ? +1 : -1;
//		               r[i] = r[i-1] + |dx|; 
//                    while (r[i] >= dy) {r[i] -= dy; x[i] += inc_x;}
//-----------------------------------------	
{
		// now, ABS(x2 - x1) >= ABS(y2 - y1)
		// Now x0 <= x1
	int16_t dy = y2 - y1;
	uint16_t dx = x2 - x1;
	uint16_t r = dx / 2, abs_dy = ABS(dy);

	int16_t inc_y = (dy >= 0) ? 1 : -1;	// True =1, False = -1
	int32_t y_int = y1;

	uint16_t Tcolor=DrawProp.TextColor;
	for (uint16_t x = x1; x <= x2; x++) {
      if (swapXY==0)	LCD_DrawPixel(x, (uint16_t) y_int, Tcolor);
      else	 					LCD_DrawPixel((uint16_t) y_int, x, Tcolor);
//-------------------------
// 			Iteration Mehod for			
//			s = dx/2;	if (dy < 0) s = -s;
//			if y_int = y1 + ((x-x1) * dy + (s)) / dx;	// from original y1 to y2
//	Use plus and minus instead of multiplication			
//---------------------------------------------------------------------------------			
			r += abs_dy;
			if (r >= dx){
//			while (r >= dx) {
				r -= dx;
				y_int += inc_y;
			}
    }
}

//-----------------------------------------	
//			y = y1 + ((x-x1) * dy + sgn(dy) * dx/2) / dx;	// from original y1 to y2
//                     -------------------------
//			When along y:   (x-x1) * dy  + z = (y-y1) * dx + my, where z = ((y-y1) * dx - m) % dy  
//     ==> i=(y-y1); inc[i] = i*dx + my = inc[i-1] + dx; x[i] = x1 + inc[i]/dy;
//-----------------------------------------	
//		==> Iteration for dy > 0: r[0] = dy/2 for dy >= dx; r[0] = -|dx|/2 for dy < dx.
//		==> Iteration:  inc_x = (dx > 0) ? +1 : -1;
//		               r[i] = r[i-1] + |dx|; 
//                    while (r[i] >= dy) {r[i] -= dy; x[i] += inc_x;}
//-----------------------------------------	
//  dx > 0: a.  x[i+1] (for r[i+1] = 0) or (x[i+1] +1) (for r[i+1] != 0) is the minimum x for y+1.
//          b.  And (x[i+1]-1) (for r[i+1] = 0) or x[i+1] (for r[i+1] != 0) is the maximum x for y.
//  dx < 0: a.  x[i+1] (for r[i+1] = 0) or (x[i+1] -1) (for r[i+1] != 0) is the maximun x for y+1.
//          b.  And (x[i+1]+1) (for r[i+1] = 0) or x[i+1] (for r[i+1] != 0) is the minimum x for y.
//-----------------------------------------	
static inline void PlusMinus_DrawLineAA(uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2)
{
   if (y1 > y2) {
			swap_u16(&x1, &x2);
			swap_u16(&y1, &y2);
    }		// Now y1 < y2
	uint16_t dy = y2 - y1;		// dy > 0
	int16_t dx = x2 - x1;
	uint16_t abs_dx = ABS(dx);
	int16_t r = (dy >= abs_dx) ? dy / 2 : -abs_dx/2;
		//	For dy < ABS(dx):  inc = -sgn(dy)*dx/2 = -dx/2;	// note, dy>0;
	int16_t inc_x = (dx >= 0) ? 1 : -1;	// True =1, False = -1
		
	int32_t xL = x1, xR = x1, x_int = x1;
	for (uint16_t y = y1; y <= y2; y++) {
		int16_t x;
//-------------------------
//	Use Addition and Subtraction instead of multiplication and division	for "inc"		
//---------------------------------------------------------------------------------	
//			inc += dx; 				// inc = (y-y1) * dx + sgn(dx) * dy/2; 
//			x = x1 + (inc) / dy;	
//-------------------------------
			r += abs_dx;
			while (r >= dy) {
				r -= dy;
				x_int += inc_x;		// 1 for dx > 0; -1 for dx < 0.
			}
			x = x_int;
				// for the present y: 
				// (if |dx| > dy) many x's per y; otherwise (|dx| < dy) only one x per y, but may be same single x for different y. 
		if ((abs_dx > dy)) {
				// z == 0 ==> max or min x for present y: max: dx> 0, xR = x-1; min: dx <0, xL = x+1
				// z != 0 ==> maxi or min x for present y: max: dx> 0, xR = x; min: dx <0, xL = x;
				//        ==> min or max x for next y=y+1: z==0: x; z!=0: dx> 0, x_min = x++; dx <0, x_max = x--;
				if (dx > 0) {
						if (r == 0) xR = x - 1;
						else				xR = x++;
						if (xR > x2) xR = x2;
				} else		{
						if (r == 0) xL = x + 1;
						else				xL = x--;
						if (xL < x2) xL = x2;
				}
		}
		LCD_DrawHLine2P( (uint16_t) xL, y, (uint16_t) xR, y);
				// for the next y = y+1
		xL = xR = x;
	}
}

//-----------------------------------------------------------------
//-----------------------------------------	
//			y = y1 + ((x-x1) * dy + sgn(dy) * dx/2) / dx;	// from original x1 to x2; dx > 0
//                     -------------------------
//		Line is shifted left by (dx/|dy|)/2 ==> new_i = i + (dx/|dy|)/2;
//			(new_i) * dy = i* dy + (dx/2) * (dy/|dy|) = i*dy + sgn(dy) * (dx/2)
//			define mX =: sgn(dy) * (dx/2);	
//			define mY =: sgn(dx) * (dy/2);	
//                     -------------------------
//									Integer Format:	
//		1. When dx >= |dy| > 0: (y-y1) * dx  + r = (x-x1) * dy + mX, where dx > r = ((x-x1) * dy + mX) % dx  
//     ==> i=(x-x1); inc[i] = i*dy + mX = inc[i-1] + dy; y[i] = y1 + inc[i]/dx; 
//		2. When 0 < |dx| < dy:  (x-x1) * dy  + z = (y-y1) * dx + mY, where dy > z = ((y-y1) * dx - mY) % dy  
//     ==> i=(y-y1); inc[i] = i*dx + mY = inc[i-1] + dx; x[i] = x1 + inc[i]/dy;
//
//	Use y direction to compute the case of |dx| > |dy| and assume dy > 0:
//      (x-x1) * dy  - r = (y-y1) * dx - mX  ==> (x-x1) * dy  - k*dy + z[i] = (y-y1) * dx - mX,
//        where 0<= z = ((y-y1) * dx - mX) % dy < dy
//              x-k is the minimum x for y. 
//      Note:  ((x-k) + dx/dy) *dy + z[i+1] = (y-y1) * dx - mX + dx = ((y+1)-y1) * dx - mX 
//          ==> Let x[i+1] = x-k+dx/dy ==> x[i+1] *dy + z[i+1] = ((y+1) - y1) - mX
// dx>0:    a.  x[i+1] (for z[i+1] = 0) or (x[i+1] +1) (for z[i+1] != 0) is the minimum x for y+1.
//          b.  And (x[i+1]-1) (for z[i+1] = 0) or x[i+1] (for z[i+1] != 0) is the maximum x for y.
// dx<0:    a.  x[i+1] (for z[i+1] = 0) or (x[i+1] -1) (for z[i+1] != 0) is the mximum x for y+1.
//          b.  And (x[i+1]+1) (for z[i+1] = 0) or x[i+1] (for z[i+1] != 0) is the minimum x for y.
//---------------------------------------------------------------------------------
static inline void Integer_DrawLineAA(uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2)
{
   if (y1 > y2) {
			swap_u16(&x1, &x2);
			swap_u16(&y1, &y2);
    }		// Now y1 < y2
	uint16_t dy = y2 - y1;		// dy > 0
	int16_t dx = x2 - x1;

	uint16_t abs_dx = ABS(dx);
	int16_t sgn_dx = (dx>0) ? +1 : -1;
	int16_t inc = (dy >= ABS(dx)) ? (sgn_dx * dy)/2 : -dx/2;
		//	For dy < ABS(dx):  inc = -sgn(dy)*dx/2 = -dx/2;	// note, dy>0;

	int32_t xL = x1, xR = x1;
	for (uint16_t y = y1; y <= y2; y++) {
		int16_t x, z;
//-------------------------
//	Use Addition and Subtraction instead of multiplication and division	for "inc"		
//---------------------------------------------------------------------------------	
		inc += dx; 				// inc = (y-y1) * dx + sgn(dx) * dy/2; 
		x = x1 + inc / dy;
				// for the present y: 
				// (if |dx| > dy) many x's per y; otherwise (|dx| < dy) only one x per y, but may be same single x for different y. 
		if ((abs_dx > dy) ) {
				z = inc % dy;
				// z == 0 ==> max or min x for present y: max: dx> 0, xR = x-1; min: dx <0, xL = x+1
				// z != 0 ==> maxi or min x for present y: max: dx> 0, xR = x; min: dx <0, xL = x;
				//        ==> min or max x for next y=y+1: z==0: x; z!=0: dx> 0, x_min = x++; dx <0, x_max = x--;
				if (dx > 0) {
						if (z == 0) xR = x - 1;
						else				xR = x++;
						if (xR > x2) xR = x2;
				} else		{
						if (z == 0) xL = x + 1;
						else				xL = x--;
						if (xL < x2) xL = x2;
				}
		}
		LCD_DrawHLine2P( (uint16_t) xL, y, (uint16_t) xR, y);
					// for the next y = y+1
		xL = xR = x;
	}
}



//-----------------------------------------------------------------
static inline void Integer_DrawLine(uint8_t swapXY, uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2)
{
		// now, ABS(x2 - x1) >= ABS(y2 - y1)
		// Now x0 <= x1
	uint16_t dx = x2 - x1;
	int16_t dy = y2 - y1;

	uint16_t y = y1;
	int32_t inc = (dy >= 0) ? dx/2: -dx/2;
	// ==> inc = (x1 - x1)*dy + sgn(dy) * dx/2;

	uint16_t Tcolor = DrawProp.TextColor;
	for (uint16_t x = x1; x <= x2; x++) {
      if (swapXY==0)	LCD_DrawPixel(x, y, Tcolor);
      else						LCD_DrawPixel(y, x, Tcolor);
//-------------------------
//	Use Addition and Subtraction instead of multiplication and division	for "inc"		
//---------------------------------------------------------------------------------		
			inc += dy; 				// inc = (x+1 - x1) * dy + sgn(dy) * dx/2; 
			y = y1 + (inc) / dx;	// y for (x+1): y = y1 + ((x+1-x1) * dy + sgn(dy) * dx/2) / dx
  }
}

//------------------------------------------------
//===============================================================
void LCD_DrawLine(uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2)
{
	uint16_t deltaX, deltaY;

	deltaX = (uint16_t)ABS(x2 - x1);        /* The difference between the x's */
  deltaY = (uint16_t)ABS(y2 - y1);        /* The difference between the y's */

	if ((deltaY == 0) || (deltaX == 0)) {
			// Horizontal Line (y1 = y2)
		uint8_t HLine = 1;
			// Vertical Line	(x1 = x2)
		if (deltaX == 0) HLine = 0;
			LCD_DrawHV2P(HLine, x1, y1, x2, y2);
			return;
	}

    // General:      (x1 != x2, otherwise 0 = deltaX = deltaY = dy = 0)
//		DrawProp.TextColor = WHITE;
		PlusMinus_DrawLineAA(x1, y1, x2, y2);		// iteration method is the quickest way
//		Integer_DrawLineAA(x1, y1, x2, y2);		// tradional method
//-------------------------------------------	
// The following methods are not recommended,
//		but are used for verifying!
//-------------------------------------------
//	uint8_t swapXY = deltaX < deltaY;
//    if (swapXY) {		// if (ABS(y1 - y0) > ABS(x1 - x0))
//			swap_u16(&x1, &y1);
//			swap_u16(&x2, &y2);
//    }	
//		// now, ABS(x2 - x1) >= ABS(y2 - y1)
//    if (x1 > x2) {
//			swap_u16(&x1, &x2);
//			swap_u16(&y1, &y2);
//    }		// Now x1 < x2, ABS(x2 - x1) >= ABS(y2 - y1)

//		DrawProp.TextColor = RED;
//		Integer_DrawLine(swapXY, x1, y1, x2, y2);		// tradional method
//		Qformat_DrawLine(swapXY, x1, y1, x2, y2);
}


/**
  * @brief  Displays a mono-color picture.
  * @param  Pict: pointer to the picture array.
  * @retval None
  */
void LCD_DrawMonoPict(const uint32_t *Pict)
{
	uint16_t fcolor=DrawProp.TextColor;
	uint16_t bcolor=DrawProp.BackColor;
  uint32_t index = 0, i = 0;

  LCD_OpenWin(0, 0, LCD_PIXEL_WIDTH, LCD_PIXEL_HEIGHT);
//  LCD_SetCursor(0, (LCD_PIXEL_WIDTH - 1));
//  LCD_WriteRAM_Prepare(); /* Prepare to write GRAM */
 
	for(index = 0; index < 2400; index++)
  {
    for(i = 0; i < 32; i++)						// 2400 x 32 = 240 x 320
    {
      if((Pict[index] & (1 << i)) == 0x00)
      {
        LCD_WriteData(bcolor);		// background color
      }
      else
      {
        LCD_WriteData(fcolor);
      }
    }
  }
}

/**
  * @brief  Draws a bitmap picture (16 bpp) (24 bpp will be transfered to 16 bpp automatically).
  * @param  Xpos: Bmp X position in the LCD
  * @param  Ypos: Bmp Y position in the LCD
  * @param  pbmp: Pointer to Bmp picture address.
  * @retval None
  */
void LCD_DrawBitmap(uint16_t Xpos, uint16_t Ypos, uint8_t *BmpAddress)	
{
  uint32_t index; 
  uint16_t bitCount;
	
	uint32_t height, i, n;
  uint32_t width, j, j_ignore;
  
  //---------------->>>>>>>>>>>>>>>
	/* Read BMP signature */
  width = *(uint16_t *) (BmpAddress + 0);
	if (width != 0x4D42) return;
	
  /* Read bitmap width */
  width = *(uint16_t *) (BmpAddress + 18);
  width |= (*(uint16_t *) (BmpAddress + 20)) << 16;
  
  /* Read bitmap height */
  height = *(uint16_t *) (BmpAddress + 22);
  height |= (*(uint16_t *) (BmpAddress + 24)) << 16; 

	//-----------------<<<<<<<<<<<<<<<<<<<<<
  
  /* Read number of bits per pixel */
  bitCount = *(uint16_t *) (BmpAddress + 28);

  /* Read bitmap size */
  /* Get bitmap data address offset */
  index = *(__IO uint16_t *) (BmpAddress + 10);
  index |= (*(__IO uint16_t *) (BmpAddress + 12)) << 16;
  BmpAddress += index;
	
//>>>######################################################
  /* Set GRAM write direction and BGR = 1 */
		LCD_WriteReg(0x36);  // Memory Access Control (36h)
  #if (LANDSCAPE == 1)
    /* BMP updates from bottom to top and left to right        */
		LCD_WriteData(0xE8);	// bit 7, 6, 5, 3 = 1  
		// bit 3 (BGR) = 1
		// bit 5 (column and row exchange) = 1
		// bit 6 (column addr. order) = 1   width ==> 0
    // bit 7 (page (row) addr. order) = 1  height ==> 0
		//
		//   column (y) 
		//   ^
		//   |
		//   |
		//   --------------------> page (x)
		//
  #else
    /* AM=0   (address is updated in horizontal writing direction)            */
		LCD_WriteData(0x48);	// bit 7, 6, 3 = 1  
		// bit 3 (BGR) = 1
		// bit 6 (column addr. order) = 1   width ==> 0
    // bit 7 (page (row) addr. order) = 0  height ==> 0
		//
		//   column (x) 
		//   ^
		//   |
		//   |
		//   --------------------> page (y)
		//
  #endif
//<<<######################################
//	LCD_DisplayWindow_WnH(Xpos, 241-(Ypos+height), width, height);
//	LCD_DisplayWindow_WnH(Xpos, LCD_PIXEL_HEIGHT-(Ypos+(height-1)), width, height);			// corrected on 2024.07.17
//>>>>>########### revised on 2024.08.01 ###############################
	i = Ypos + height - 1;			// Y1
	n = 0;
	if (i > LCD_PIXEL_HEIGHT){
		n = i - LCD_PIXEL_HEIGHT;			// ignore the image part on the last "n" lines
		i = LCD_PIXEL_HEIGHT;
	}
	j = Xpos + width - 1;			// X1
	j_ignore = width;
	if (j > LCD_PIXEL_WIDTH){
		if (Xpos > LCD_PIXEL_WIDTH) j_ignore = 0;
		else j_ignore = (LCD_PIXEL_WIDTH+1) - Xpos;
		j = LCD_PIXEL_WIDTH;
	}
	LCD_OpenWin(Xpos, LCD_PIXEL_HEIGHT-i,  j,  LCD_PIXEL_HEIGHT-Ypos);	// i.e., LCD_OpenWin(X0, Y0,  X1,  Y1);
//<<<<<########### revised on 2024.08.01 ###############################

  if (bitCount == 24)
	{
		uint8_t skip;
		uint16_t n_ignore = width - j_ignore;		//#### revised on 2024.08.01
		
		skip = 0x03 & (4 - ( (width*3) &0x03));   // align in 4 multiple
		if (n > 0){																// ignore "n" lines
			BmpAddress += n * (3*width + skip);					//#### revised on 2024.08.01
		}
			
		for (i = n; i<height; i++)				
		{ 
			for (j=0; j<j_ignore; j++)	//#### revised on 2024.08.01
			{
				uint16_t temp;
/*				uint16_t t2;
				t2 = (*(__IO uint8_t *)BmpAddress++);
				temp = t2 >>3;
				t2 = (*(__IO uint8_t *)(BmpAddress++));
				temp |= (t2 >>3)<<6;
				t2 = (*(__IO uint8_t *)(BmpAddress++));
				temp |= (t2 >>3)<<(5+6);
*/
					temp = (*(__IO uint8_t *)BmpAddress++)>>3;
					temp |= ((*(__IO uint8_t *)(BmpAddress++) )>> 3)  << 6;
					temp |= ((*(__IO uint8_t *) (BmpAddress++) )>> 3)  << (6+5);
					LCD_WriteData(temp);
			}
			BmpAddress += 3*n_ignore;					//#### revised on 2024.08.01
			BmpAddress += skip;
		}
	}
	else // bitCount == 16 [RED (5bits): GREEN (6bits): BLUE (5bits)]
	{
//>>>>>########### revised on 2024.08.01 ###############################
		uint16_t n_ignore = width - j_ignore;		

		if (n > 0){				// ignore "n" lines		
			BmpAddress += n * (2*width);
		}
		
		for (i = n; i<height; i++)				
		{ 
			for (j=0; j<j_ignore; j++)	//#### revised on 2024.08.01
			{
				LCD_WriteData(*(__IO uint16_t *)BmpAddress);
				BmpAddress += 2;
			}
			BmpAddress += 2 * n_ignore;					//#### revised on 2024.08.01
		}
//<<<<<########### revised on 2024.08.01 ###############################
  }

//>>>######################################################
  /* Set GRAM write direction and BGR = 1 */
		LCD_WriteReg(0x36);  // Memory Access Control (36h)
  #if (LANDSCAPE == 1)
		LCD_WriteData(0xA8);	// bit 7, 5, 3 = 1  
		// bit 3 (BGR) = 1
		// bit 5 (column and row exchange) = 1
		// bit 6 (column addr. order) = 0   0 ==> width 
    // bit 7 (page (row) addr. order) = 1  height ==> 0
		//
		//   --------------------> page (x)
		//   |
		//   |
		//   \/
		//   column (y) 
		//
  #else
		LCD_WriteData(0xC8);	// bit 7, 6, 3 = 1  
		// bit 3 (BGR) = 1
		// bit 6 (column addr. order) = 1   width ==> 0
    // bit 7 (page (row) addr. order) = 1  height ==> 0
		//
		//   column (x) 
		//   ^
		//   |
		//   |
		//   --------------------> page (y)
		//
  #endif
//<<<######################################
}

//---------------------------------
static void LCD_DrawRGBImage_2P(uint16_t x0, uint16_t y0, uint16_t x1, uint16_t y1, uint8_t *pdata)
{
	if (!pdata) {
		LCD_FillRect2P(x0, y0, x1, y1);
		return;
	}
	LCD_OpenWin(x0, y0, x1, y1);  

  for(uint16_t j= y0; j <= y1; j++)
		for(uint16_t i= x0; i <= x1; i++)
		{
			uint16_t pixel = *(volatile uint16_t *)pdata;
			LCD_WriteData(pixel);
			pdata += 2;
		}
}
/**
  * @brief  Draws RGB Image (16 bpp).
  * @param  Xpos:  X position in the LCD
  * @param  Ypos:  Y position in the LCD
  * @param  Xsize: X size in the LCD
  * @param  Ysize: Y size in the LCD
  * @param  pdata: Pointer to the RGB Image address.
  * @retval None
  */
void LCD_DrawRGBImage(uint16_t Xpos, uint16_t Ypos, uint16_t Xsize, uint16_t Ysize, uint8_t *pdata)
{
  uint16_t x1, y1;
  
	if ( (Xpos > LCD_PIXEL_WIDTH) || (Ypos > LCD_PIXEL_HEIGHT)) return;
	x1 = Xpos + Xsize-1;
	y1 = Ypos + Ysize-1;
	if (x1 > LCD_PIXEL_WIDTH) x1 = LCD_PIXEL_WIDTH;
	if (y1 > LCD_PIXEL_HEIGHT) y1 = LCD_PIXEL_HEIGHT;

	LCD_DrawRGBImage_2P(Xpos, Ypos, x1, y1, pdata);
}

//====================================================
void LCD_DrawRGBbuffer(sImageBuf *pImageSt){
  uint16_t X0, Y0, X_most, Y_most, width, height;
 	
		DecomposeImStr(&X0, &Y0, &width, &height, &X_most, &Y_most, pImageSt);
		uint8_t *dataBuf = (uint8_t *) pImageSt->data;
		LCD_DrawRGBImage_2P(X0, Y0, X_most, Y_most, dataBuf);
}

void LCD_RectBuf_or_RectBack_skipRect(sImageBuf *imageSt, Point *skip_p, Point *Back_p);
//====================================================
void LCD_DrawRGBbuffer_skipRect(sImageBuf *pImageSt, Point *skip_p){
	LCD_RectBuf_or_RectBack_skipRect(pImageSt, skip_p, 0);
}

//====================================================
void LCD_FillRectBack_skipRect(Point *DrawB_p, Point *skip_p){
	LCD_RectBuf_or_RectBack_skipRect(0, skip_p, DrawB_p);
}

//====================================================
void LCD_RectBuf_or_RectBack_skipRect(sImageBuf *pImageSt, Point *skip_p, Point *Back_p){
  uint16_t X0, Y0, X_most, Y_most, width=0, height;
  uint16_t *pBuffer;
  uint16_t SKx0=0, SKy0=0, SKx1=0, SKy1=0;
  uint16_t Tcolor;
	
	if ((pImageSt == 0) && (Back_p==0)) return;
	if (pImageSt) {
		DecomposeImStr(&X0, &Y0, &width, &height, &X_most, &Y_most, pImageSt);
		pBuffer = pImageSt->data;
	} else { // Back_p != 0: Draw the background color on the rectangle range: Back_p[0] ~ Back_p[2]
			X0 = Back_p[0].X; 
			Y0 = Back_p[0].Y;
			X_most = Back_p[1].X; 
			Y_most = Back_p[1].Y;	
			if	((X_most < X0) || (Y_most < Y0)  ) 	return;
			Tcolor = DrawProp.BackColor;
			DrawProp.TextColor = DrawProp.BackColor;
			pBuffer = 0;
	}
	
	uint8_t w1 = 1;
	if (skip_p){		// having skip rectangle:  skip_p[0] ~ skip_p[2]
			SKx0 = skip_p[0].X; 
			SKy0 = skip_p[0].Y;
			SKx1 = skip_p[1].X; 
			SKy1 = skip_p[1].Y;	
		if	((SKx1 < SKx0) || (SKy1 < SKy0)  ) 	return;
	//-------------------------
	//	make sure that
	//	X0 <= SKx0 <= SKx1 <= X_most
	//	Y0 <= SKy0 <= SKy1 <= y_most
	//	min_y1 = min(y1, SKy1) = SKy1
	//-------------------------
		if (adjust_x1_less_x2_inRange(&SKx0, &SKx1, X0, X_most) ) {
			adjust_x1_less_x2_inRange(&SKy0, &SKy1, Y0, Y_most);
		} else w1 = 0;
	} else w1 = 0; 	// when skip_p = 0, no skip rectangle
	
			// no skip rectangle
	uint8_t *dataBuf = (uint8_t *) pBuffer;
	if (w1 == 0){		// no intersection (overlapping)
		LCD_DrawRGBImage_2P(X0, Y0, X_most, Y_most, dataBuf);
		if (!pImageSt) 	DrawProp.TextColor = Tcolor;
		return;
	}

			// having skip rectangle
	//====== Part 1: Y0 <= .... < SKy0
	if (Y0 < SKy0){
		LCD_DrawRGBImage_2P(X0, Y0, X_most, SKy0-1, dataBuf);	// (x0, y0) ~ (x1, SKy0-1)
		if (pImageSt) pBuffer += width * (SKy0 - Y0);
	}

	//=== Part 2: SKy0 <= ..... <= SKy1 (= min(SKy1, Y_most) = min_y1), since Y_most >= SKy1 definitely (if (SKy1 > Y_most) SKy1 = Y_most;	)
	// Note: Here always Y0 <= SKy0 <= SKy1 <= Y_most
//	w1 = SKx0 - X0;		// w1 > 0
	uint16_t i = 0, x0, x1;
		if (SKx0 > X0){ 
		//---------- Left subpart: x0 <= ... < SKx0
			x0 = X0; 
			x1 = SKx0-1;
			if (pImageSt)	dataBuf = (uint8_t *) pBuffer;
		} else i = 1; //i=1: not do left subpart
	for (; i <= 1; i++) {
		if (i == 1){
		//----------- Right subpart: SKx1 < ... <=x1 (NOT include SKx1)
			if (X_most <= SKx1) break;
			x0 = SKx1+1;
			x1 = X_most;
			if (pImageSt) dataBuf = (uint8_t *) pBuffer + 2 * (x0 - X0);
		}			

		if (pImageSt){	
				for(uint16_t y= SKy0; y <= SKy1; y++)
				{	
					LCD_DrawRGBImage_2P(x0, y, x1, y, dataBuf);
					dataBuf += 2 * width;
				}
		} else 		LCD_FillRect2P(x0, SKy0, x1, SKy1); 	// (x0, SKy0) ~ (x1, SKy1)
	}
	

	//====== Part 3: min_y1+1 (= SKy1+1) <= .... <= Y_most
	if (Y_most > SKy1){
		if (pImageSt) pBuffer += width * (SKy1 - SKy0 + 1);  // height = (SKy1 - SKy0 + 1)
		LCD_DrawRGBImage_2P(X0, (SKy1+1), X_most, Y_most,  (uint8_t *) pBuffer);
	}
	
	if (!pImageSt) 	DrawProp.TextColor = Tcolor;
}


//============ added on 07.03. 2025 ==========================
void LCD_DrawRGB_Transparent(uint16_t Xpos, uint16_t Ypos, uint16_t Xsize, uint16_t Ysize, uint8_t *pdata, uint16_t transColor)
{
  uint16_t x1, y1;
  uint16_t i, j;
	uint8_t transTrue = 0;	// not a transparent pixel
//	#define transColor WHITE  
		uint8_t	updated_2C=0;
	
	if ( (Xpos > LCD_PIXEL_WIDTH) || (Ypos > LCD_PIXEL_HEIGHT)) return;
	x1 = Xpos + Xsize-1;
	y1 = Ypos + Ysize-1;
	if (x1 > LCD_PIXEL_WIDTH){
		Xsize = LCD_PIXEL_WIDTH+1 - Xpos;
		x1 = LCD_PIXEL_WIDTH;
	}
	if (y1 > LCD_PIXEL_HEIGHT){
		Ysize = LCD_PIXEL_HEIGHT+1 - Ypos;
		y1 = LCD_PIXEL_HEIGHT;
	}
  
  LCD_DisplayWindow_WnH(Xpos, Ypos, Xsize, Ysize);

  for(j= 0; j < Ysize; j++)
  {
		uint16_t pixel;
		uint16_t xx, yy;
		
		if (updated_2C == 1){
			updated_2C = 0;
			transTrue = 1;
		}

		for(i= 0; i < Xsize; i++)
		{
			pixel = *(volatile uint16_t *)pdata;
			if (pixel != transColor){
				if ((transTrue == 1)){
					xx = i+Xpos;
					LCD_WriteReg(0x2A);					// Column Address Set (2Ah)
					LCD_WriteData(xx>>8);
					LCD_WriteData(0x00FF&xx);		
					LCD_WriteData(x1>>8);
					LCD_WriteData(0x00FF&x1);
					if (updated_2C == 0){
						yy = j + Ypos;
						LCD_WriteReg(0x2B);				// Page (Row) Address Set (2Bh) 
						LCD_WriteData(yy>>8);
						LCD_WriteData(0x00FF&yy);		
						LCD_WriteData(y1>>8);
						LCD_WriteData(0x00FF&y1);
					}
					LCD_WriteReg(0x2C);				// Write Data to GRAM (2Ch) 
					// NOTE:  Hereafer, the column register and the page register are reset 
					//        to the START Column/START Page positions.
					updated_2C = 1;

					transTrue = 0;
				}
    /* Write 16-bit GRAM Reg */
				LCD_WriteData(pixel);
			} else {
					transTrue = 1;
			}
			pdata += 2;
		}
	}
}

/**
  * @brief  Displays an poly-line (between many points).
  * @param  Points: pointer to the points array.
  * @param  PointCount: Number of points.
  * @retval None
  */
void LCD_PolyLineClosed(pPoint p, uint16_t pCount, uint8_t Closed)
{
  uint16_t X, Y;
  pPoint First = p;

  if(pCount < 2)
  {
    p++;
    X = p->X;
    Y = p->Y;
    LCD_DrawLine(First->X, First->Y, X, Y);
    return;
  }

  while(--pCount)
  {
		uint16_t x0, y0;
    x0 = p->X;
    y0= p->Y;
    p++;
    X = p->X;
    Y = p->Y;
    LCD_DrawLine(x0, y0, X, Y);
	}
 
  if(Closed)   LCD_DrawLine(First->X, First->Y, X, Y);
}

/**
  * @brief  Displays an relative poly-line (between many points).
  * @param  Points: pointer to the points array.
  * @param  PointCount: Number of points.
  * @param  Closed: specifies if the draw is closed or not.
  *           1: closed, 0 : not closed.
  * @retval None
  */
static void LCD_PolyLineRelativeClosed(pPoint Points, uint16_t PointCount, uint8_t Closed)
{
  uint16_t X = 0, Y = 0;
  pPoint First = Points;

  if(PointCount < 2)
  {
    return;
  }
  X = Points->X;
  Y = Points->Y;
  while(--PointCount)
  {
    Points++;
    LCD_DrawLine(X, Y, X + Points->X, Y + Points->Y);
    X = X + Points->X;
    Y = Y + Points->Y;
  }
  if(Closed)   LCD_DrawLine(First->X, First->Y, X, Y);
}

/**
  * @brief  Displays a closed poly-line (between many points).
  * @param  Points: pointer to the points array.
  * @param  PointCount: Number of points.
  * @retval None
  */
//void LCD_DrawPolygon(pPoint Points, uint16_t PointCount)
//{
////  LCD_PolyLine(Points, PointCount);
////  LCD_DrawLine(Points->X, Points->Y, (Points+PointCount-1)->X, (Points+PointCount-1)->Y);
//	LCD_PolyLineClosed(Points, PointCount, 1);
//}

/**
  * @brief  Displays a relative poly-line (between many points).
  * @param  Points: pointer to the points array.
  * @param  PointCount: Number of points.
  * @retval None
  */
void LCD_PolyLineRelative(pPoint Points, uint16_t PointCount)
{
  LCD_PolyLineRelativeClosed(Points, PointCount, 0);
}

/**
  * @brief  Displays a closed relative poly-line (between many points).
  * @param  Points: pointer to the points array.
  * @param  PointCount: Number of points.
  * @retval None
  */
void LCD_ClosedPolyLineRelative(pPoint Points, uint16_t PointCount)
{
  LCD_PolyLineRelativeClosed(Points, PointCount, 1);
}

/**
  * @brief  Draws an ellipse on LCD.
  * @param  Xpos: X position
  * @param  Ypos: Y position
  * @param  XRadius: Ellipse X radius
  * @param  YRadius: Ellipse Y radius
  * @retval None
  */
void LCD_DrawEllipse(int Xpos, int Ypos, int XRadius, int YRadius)
{
  int x = 0, y = -YRadius, err = 2-2*XRadius, e2;
  float K = 0, rad1 = 0, rad2 = 0;
  
  rad1 = (float) XRadius;
  rad2 = (float) YRadius;
  
  K = (float)(rad2/rad1);
  
  do {      
    LCD_DrawPixel((Xpos-(uint16_t)(x/K)), (Ypos+y), DrawProp.TextColor);
    LCD_DrawPixel((Xpos+(uint16_t)(x/K)), (Ypos+y), DrawProp.TextColor);
    LCD_DrawPixel((Xpos+(uint16_t)(x/K)), (Ypos-y), DrawProp.TextColor);
    LCD_DrawPixel((Xpos-(uint16_t)(x/K)), (Ypos-y), DrawProp.TextColor);      
    
    e2 = err;
    if (e2 <= x) {
      err += ++x*2+1;
      if (-y == x && e2 <= y) e2 = 0;
    }
    if (e2 > y) err += ++y*2+1;     
  }
  while (y <= 0);
}


/**
  * @brief  Fills a triangle (between 3 points).
  * @param  Points: Pointer to the points array
  * @param  Points[0].x: Point 1 X position
  * @param  Points[0].y: Point 1 Y position
  * @param  Points[1].x: Point 2 X position
  * @param  Points[1].y: Point 2 Y position
  * @param  Points[2].x: Point 3 X position
  * @param  Points[2].y: Point 3 Y position
  * @retval None
  */
// revised on 2026/01/02
 void LCD_FillTriangle_RGBbuffer3P(pPoint P0, pPoint P1, pPoint P2, sImageBuf *pImageSt);

void LCD_FillTriangle(pPoint Points)
{
		LCD_FillTriangle_RGBbuffer3P(&Points[0], &Points[1], &Points[2], 0);
}
//===================================================================
void LCD_FillTriangle_RGBbuffer(pPoint Points, sImageBuf *pImageSt)
{
		LCD_FillTriangle_RGBbuffer3P(&Points[0], &Points[1], &Points[2], pImageSt);
}


//================================================================
/**
  * @brief  Draws a full poly-line (between many points).
  * @param  Points: Pointer to the points array
  * @param  PointCount: Number of points
  * @retval None
  */
void LCD_FillPolygon(pPoint Points, uint16_t PointCount)
{
  int16_t i;

  if(PointCount <=1) return;
  if(PointCount ==2) {
		uint16_t x, y;
    x = Points->X;
    y = Points->Y;
    Points++;
    LCD_DrawLine(x, y, Points->X, Points->Y);
		return;
  }

  for(i = 0; i < (PointCount-2); i++)
  {
		LCD_FillTriangle_RGBbuffer3P(&Points[0], &Points[i+1], &Points[i+2], 0);
  }
}

void LCD_DrawRGBbuffer_Transparent(sImageBuf *pImageSt, uint16_t transColor){
  uint16_t X0, Y0, width, height;
  uint8_t *pdataBuf;
	
	X0 = pImageSt->topLeft.X;
	Y0 = pImageSt->topLeft.Y;
	width = pImageSt->width;
	height = pImageSt->height;
	pdataBuf = (uint8_t *) pImageSt->data;
	LCD_DrawRGB_Transparent(X0, Y0, width, height, pdataBuf, transColor);
}

//================================================================
void LCD_FillPolygon_RGBbuffer(pPoint Points, uint16_t PointCount, sImageBuf *pImageSt)
{
  int16_t i;

  if(PointCount <=1) return;
  if(PointCount ==2)
  {
		uint16_t x, y;
    x = Points->X;
    y = Points->Y;
    Points++;
    LCD_DrawLine_RGBbuffer(x, y, Points->X, Points->Y, pImageSt);
		return;
  }

  for(i = 0; i < (PointCount-2); i++)
  {
		LCD_FillTriangle_RGBbuffer3P(&Points[0], &Points[i+1], &Points[i+2], pImageSt);
	}
}

/**
  * @brief  Draws a full ellipse.
  * @param  Xpos: X position
  * @param  Ypos: Y position
  * @param  XRadius: Ellipse X radius
  * @param  YRadius: Ellipse Y radius  
  * @retval None
  */
void LCD_FillEllipse(int Xpos, int Ypos, int XRadius, int YRadius)
{
  int x = 0, y = -YRadius, err = 2-2*XRadius, e2;
  float K = 0, rad1 = 0, rad2 = 0;
  
  rad1 = XRadius;
  rad2 = YRadius;
  
  K = (float)(rad2/rad1);    
  
  do 
  { 
    LCD_DrawHLine((Xpos-(uint16_t)(x/K)), (Ypos+y), (2*(uint16_t)(x/K) + 1));
    LCD_DrawHLine((Xpos-(uint16_t)(x/K)), (Ypos-y), (2*(uint16_t)(x/K) + 1));
    
    e2 = err;
    if (e2 <= x) 
    {
      err += ++x*2+1;
      if (-y == x && e2 <= y) e2 = 0;
    }
    if (e2 > y) err += ++y*2+1;
  }
  while (y <= 0);
}


//============== NEW by Shir-Kuan Lin ++++++++++++
sFONT *lFont;
uint16_t	ltColor, lbColor;

//----------------------
void LCD_SaveColors(void)
{
	ltColor = DrawProp.TextColor;
	lbColor = DrawProp.BackColor;
}
//----------------------
void LCD_RestoreColors(void)
{
	DrawProp.TextColor = ltColor;
	DrawProp.BackColor = lbColor;
}
//----------------------
void LCD_SaveFont(void)
{
	lFont = pLCD_Currentfonts;
}
//----------------------
void LCD_RestoreFont(void)
{
  LCD_SetFont(lFont);
}
//---------------------------
// if no tail string "ptr": Use (char*)'\n' instead.
// For Example: LCD_digital(row_no, col_no, val,  (char*)'\n');
//---------------------------------
void LCD_digital(uint8_t row_no, uint8_t col_no, int16_t val, char *ptr)
{
  char p_text[6] = "";

	sprintf(p_text, "%i%s", val, ptr);  //% read a decimal, octal, or hexadecimal integer
	LCD_DisplayStringLineCol(row_no, col_no, p_text);
}

//########################################################
//########################################################
//  Test  Experiments 													 
//########################################################
//########################################################
void LCD_FillTriangleAA(Point p[3]);

/**
  * @brief  Test LCD Display
  * @retval None
  */
void LCD_RGB_Test(void)
{
  uint32_t index, p1, p;

  LCD_OpenWin(0, 0, LCD_PIXEL_WIDTH, LCD_PIXEL_HEIGHT);
	p = (LCD_PIXEL_HEIGHT+1)/3;
	p = p * (LCD_PIXEL_WIDTH+1);

	/* R */
  for(index = 0; index <= p; index++)
  {
    LCD_WriteData(LCD_COLOR_RED);
  }
	  
  /* G */
	p = 2 * p;
  for(; index <= p; index++)
  {
    LCD_WriteData(LCD_COLOR_GREEN);
  }
	  
	/* B */
	p = (LCD_PIXEL_HEIGHT+1)*(LCD_PIXEL_WIDTH+1);
  for(; index <= p; index++)
  {
    LCD_WriteData(LCD_COLOR_BLUE);
  }

	p1 = (DrawProp.pFont)->Height; // font_height in pixel;	
	p1 = ((LCD_PIXEL_HEIGHT+1) +(p1-1)) /p1;
	LCD_SetColors(WHITE, RED);
  LCD_DisplayStringLineCol(2, 1,"R"); // line 2, column 1
	p = p1/3;
	LCD_SetBackColor(GREEN);
  LCD_DisplayStringLineCol((uint8_t) (2+p), 1,"G"); // line , column 1
	p = (2*p1)/3;
	LCD_SetBackColor(BLUE);
  LCD_DisplayStringLineCol((uint8_t) (2+p), 1,"B"); // line , column 1
	  delay_ms(1000); /* delay 1000 ms */
}


//==========================
// LCD display off/on test 
//==========================
void LCD_DisplayOnTest(void)
{
	LCD_DisplayOff();

	delay_ms(3000);
 	//---- turn on Display ----------
	LCD_DisplayOn();
}


/**
  * @brief  Menu Initialisation routine
  */
#define point_Count 3
#define X_c	160
#define Y_c 215
#define H_R 9
void MenuInit(void)
{
  uint16_t wd;
	Point Points0[point_Count]={ 
		{(X_c-H_R+1), (Y_c-H_R)}, // Upper Left 
		{(X_c+H_R+1), (Y_c)}, 		// Right Center
		{(X_c-H_R+1), (Y_c+H_R)}, // Down Left
	};
	Point Points1[point_Count]={ 
//		{(X_c+2*H_R+1), (Y_c-H_R)}, // Upper Right
////		{(X_c+H_R+1), (Y_c)}, 		// Left Center
////		{(X_c+2*H_R+1), (Y_c+H_R)}, // Down Right
		{180, 210}, {210, 214}, {226, 226}
	};

	
	LCD_RGB_Test();
	delay_ms(200);

	/*	save current Font, Back Color*/
	LCD_SaveColors();

	LCD_SaveFont();
//	LCD_SetFont(&Font20);
  LCD_Clear(LCD_COLOR_BLUE2);
	LCD_SetColors(LCD_COLOR_MAGENTA, LCD_COLOR_BLUE2); // Test = white; back = blue

	LCD_DisplayStringLineCol(0, 2,"PARADIGM LCD DEMO");		// line 0, column 2
  LCD_SetTextColor(LCD_COLOR_GREEN);
  LCD_DisplayStringLineCol(2, 1,"Watch LEDs flashing"); // line 2, column 1

	//----------------------------------
	if (LANDSCAPE == 0) wd = 184;
	else	wd = 220;
  LCD_SetTextColor(LCD_COLOR_YELLOW);
  /*Draw a rectangle with: Start X-Cood,  Start Y-Cood,  Width,  Heigt*/
  LCD_DrawRect(50,80,wd,120);
  LCD_SetTextColor(LCD_COLOR_CYAN);
  /*Draw a rectangle with: Start X-Cood,  Start Y-Cood,  Width,  Heigt*/
  LCD_DrawRect(45,75,wd+10,130);
  /*Draw a triangle with: Start X-Cood,  Start Y-Cood,  Width,  Heigt*/
		LCD_FillTriangle(Points0);
		LCD_DrawPolygon(Points1,  3);
		LCD_SetTextColor(RED);
		LCD_FillTriangle(Points1);
		LCD_SetTextColor(WHITE);
		LCD_DrawPolygon(Points0,  3);
		LCD_DrawPolygon(Points1,  3);
	//----------------------------------
  LCD_SetTextColor(LCD_COLOR_RED);
	LCD_FillRect(53, 83, wd-6, 114);
	LCD_SetTextColor(LCD_COLOR_WHITE);
  LCD_DisplayStringLineCol(5, 4, "Value:");
	LCD_DrawCircle(160, 160, 30);
  LCD_SetTextColor(LCD_COLOR_GREEN);
	LCD_FillCircle(160, 160, 26);


	/*	restore last Font, Back Color*/
	LCD_RestoreFont();
	LCD_RestoreColors();
	
	delay_ms(2000);
	ReverseLCD();
	delay_ms(4000);
	NormalLCD();

	delay_ms(2000);
	LCD_DisplayOnTest();
}


/******************* (C) COPYRIGHT 2011 STMicroelectronics *****END OF FILE****/


/**
  * @brief  Reads RGB Image (16 bpp).
  * @param  Xpos:  X position in the LCD
  * @param  Ypos:  Y position in the LCD
  * @param  Xsize: X size in the LCD
  * @param  Ysize: Y size in the LCD
  * @param  pdata: Pointer to the RGB Image address.
  * @retval None
  */
void LCD_ReadRGBImage(uint16_t Xpos, uint16_t Ypos, uint16_t Xsize, uint16_t Ysize, uint8_t *pdata)
{
  uint16_t index, size;
  uint16_t x1, y1;
	volatile uint16_t data1, data2;
  
	if (Xpos > LCD_PIXEL_WIDTH){
		Xpos = LCD_PIXEL_WIDTH;
		if (Ypos > LCD_PIXEL_HEIGHT) return;
	}
	if (Ypos > LCD_PIXEL_HEIGHT) Ypos = LCD_PIXEL_HEIGHT;
	x1 = Xpos + Xsize-1;
	y1 = Ypos + Ysize-1;
	if (x1 > LCD_PIXEL_WIDTH) Xsize = LCD_PIXEL_WIDTH+1 - Xpos;
	if (y1 > LCD_PIXEL_HEIGHT) Ysize = LCD_PIXEL_HEIGHT+1 - Ypos;
  LCD_DisplayWindow_WnH(Xpos, Ypos, Xsize, Ysize);

	LCD_WriteReg(0x2E);	// 0x2E: Memory Read
	//delay_ms(1);
  size = (Xsize * Ysize);
	LCD_ReadData;   // the 1st return value is "dummy"

  for(index = 0; index < (size/2); index++)
	{
 		data1 = LCD_ReadData;							// data1[15:11] = RED1[4:0]; data1[7:2] = GREED1[5:0]
		data1 = (data1 & 0xF800) | ((data1<<3) & 0x07E0);
 		data2 = LCD_ReadData;							// data2[15:11] = BLUE1[4:0]; data1[7:3] = RED2[4:0]
		
 		*((volatile uint16_t *) pdata)= data1 | (data2>>11);
     pdata += 2;
		data1 = (data2<<8) & 0xF800;
 		data2 = LCD_ReadData;							// data2[15:10] = GREEN2[5:0]; data1[7:3] = BLUE2[4:0]
		data1 = data1 | ((data2>>5) & 0x07E0) | ((data2>>3) & 0x001F);
 		*((volatile uint16_t *) pdata)= data1;
     pdata += 2;
	}
	
	if( size & 0x01 )	// size is an odd integer
	{
 		data1 = LCD_ReadData;
		data1 = (data1 & 0xF800) | ((data1<<3) & 0x07E0);
 		data2 = LCD_ReadData;
		
 		*((volatile uint16_t *) pdata)= data1 | (data2>>11);
	}
}

//?????????????????
//################## Low LEVEL DRIVER
/**
  * @brief  Reads the selected LCD Register.
  * @param  LCD_Reg: address of the selected register.
  * @retval LCD Register Value.
  */

//=========== 2026.01.02 ===========================
//--------------------------------------------------
static void Swap(Point* a, Point* b)
{
    Point t = *a;
    *a = *b;
    *b = t;
}

//------------------------------------------------
static void parameters_flatTri(uint16_t dyL, int16_t dxL, uint16_t *abs_dxL, int16_t *rL, int16_t *inc_xL)
{
	*abs_dxL = ABS(dxL);
	*rL = (dyL >= *abs_dxL) ? dyL / 2 : -*abs_dxL/2;
	*inc_xL = (dxL >= 0) ? 1 : -1;	
}	
//-----------------------------------------------------
static int16_t r_And_x_int(int16_t *r, int16_t *x_int, uint16_t dy, uint16_t abs_dx, int16_t inc_x)			
{
		*r += abs_dx;
		while (*r >= dy) {
			*r -= dy;
			*x_int += inc_x;		// 1 for dxL > 0; -1 for dxL < 0.
		}
		return *x_int;
}

//----------------------------------------------------------
static void flatTop_parameters(uint16_t *dy, int16_t *dx, uint16_t *x_lmt, uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2)	
{
			*dy = y2 - y1;  // dyL > 0
			*dx = x2 - x1;
			*x_lmt = x2;
}
	//====== Flat Bottom triangle 
	//      (x0, y0) o 
	//                 \   \
	//        (x1, y1) o----o (x2, y2)
	//====== Flat Top triangle 
	//      (x0, y0) o----o (x1, y1)
	//                 \  \
	//                      o (x2, y2)
static void flatTriangles(Point v0, Point v1, Point v2)
{ // v1.Y = v2.Y > v0.Y
	uint16_t y0 = v0.Y, y1 = v1.Y, y2 = v2.Y;		// y0 < y1 <= y2 
	uint16_t x0 = v0.X, x1 = v1.X, x2 = v2.X;	// x1 <= x2
	
	uint16_t dyL, dyR;
	int16_t dxL, dxR;
	int16_t xL, xR, xL_int, xR_int;
	uint16_t xL_lmt, xR_lmt;
	uint8_t left_Line01;
	uint16_t y, ys = y0, ye;
	xL = xL_int = x0; 
	xR = xR_int = x0;
	dyR = y2 - y0;  // dyR > 0
	dxR = x2 - x0;

	if (y0 == y1) {
			//-- Flat Top triangle 
		// x1 < x0: Left side = Line 12; Right side = Line 02
		flatTop_parameters(&dyL, &dxL, &xL_lmt, x1, y1, x2, y2);
		xR_lmt = xL_lmt;
		xL = xL_int = x1;
		ye = y2;		// flat top: y0 ~ y2, y1 = y0;
	} else {		// if y0 != y1
			//-- Flat Bottom triangle 
		ye = (y2 > y1) ? y1-1 : y1;		// flat bottom: y0 ~ (y1-1); flat top: y1 ~ y2;
			dyL = y1 - y0;
			dxL = x1 - x0;
			int32_t L_slope = (dxL << 16) / dyL;
			int32_t R_slope = (dxR << 16) / dyR;
		if (L_slope == R_slope){
			LCD_DrawLine(x0, y0, x2, y2);		// a single general line instead
			return;
		} 		
		if (L_slope > R_slope){
			// Left side = Line 02; Right side = Line 01 
			left_Line01 = 0;
			swap_u16(&dyL, &dyR);
			swap_i16(&dxL, &dxR);
			xR_lmt = x1;
			xL_lmt = x2;
		} else {
			// Left side = Line 01; Right side = Line 02 
			left_Line01 = 1;
			xR_lmt = x2;
			xL_lmt = x1;
		}
	}

	uint16_t abs_dxL, abs_dxR;
	int16_t rL, rR, inc_xL, inc_xR;
	parameters_flatTri(dyL, dxL, &abs_dxL, &rL, &inc_xL);
	parameters_flatTri(dyR, dxR, &abs_dxR, &rR, &inc_xR);

loop_start:
		for (y=ys; y<=ye; y++)
		{
			int16_t tL, tR;
			tL = r_And_x_int(&rL, &xL_int, dyL, abs_dxL, inc_xL);			
			if (abs_dxL > dyL) {
				// r == 0 ==> dx> 0, xR = x-1;     dx <0, xL = x+1
				// r != 0 ==> dx> 0, xR = x, x++;  dx <0, xL = x, x--;
				if (dxL >= 0) {
						if (rL != 0) tL++;
				} else	{	// (dxL < 0)
						if (rL == 0) xL = tL + 1;
						else				 xL = tL--;
						if (xL < xL_lmt) xL = xL_lmt;
				}
			}

			tR = r_And_x_int(&rR, &xR_int, dyR, abs_dxR, inc_xR);			
			if (abs_dxR > dyR) {
				// r == 0 ==> dx> 0, xR = x-1;     dx <0, xL = x+1
				// r != 0 ==> dx> 0, xR = x, x++;  dx <0, xL = x, x--;
				if (dxR >= 0) {
						if (rR == 0) xR = tR - 1;
						else				 xR = tR++;
						if (xR > xR_lmt) xR = xR_lmt;
				} else {		// (dxR < 0)
						if (rR != 0) tR--;
				}
			}
			
			LCD_DrawHLine2P( xL, y, xR, y);		// horizontal line
			xL = tL; xR = tR;
    }
//loop_end:
		if (y > y2) return;
		
			//-- Flat Top triangle 
		ys = y1; ye = y2;

		if(left_Line01) {
			// Left side is update as Line 12;
			flatTop_parameters(&dyL, &dxL, &xL_lmt, x1, y1, x2, y2);
			parameters_flatTri(dyL, dxL, &abs_dxL, &rL, &inc_xL);
			xL = xL_int = x1;
		} else {
			// Right side is update as Line 12;
			flatTop_parameters(&dyR, &dxR, &xR_lmt, x1, y1, x2, y2);
			parameters_flatTri(dyR, dxR, &abs_dxR, &rR, &inc_xR);
			xR = xR_int = x1;
		}
		goto loop_start;
}

//==========================
/**
  * @brief  Fills a triangle (between 3 points).
  * @param  Points: Pointer to the points array
  * @retval None
  */
void LCD_FillTriangleAA(Point p[3])
{
    Point v0 = p[0];
    Point v1 = p[1];
    Point v2 = p[2];

    // Sort by Y s.t. v0.Y <= v1.Y <= v2.Y
    if (v0.Y > v1.Y) Swap(&v0, &v1);
    if (v1.Y > v2.Y) Swap(&v1, &v2);	// ==> v2.Y is the largest
    if (v0.Y > v1.Y) Swap(&v0, &v1);	// ==> v0.Y is the smallest
	uint16_t	x0, y0, x1, y1, x2, y2;
		x0 = v0.X;
		y0 = v0.Y;
		x1 = v1.X;
		y1 = v1.Y;
		x2 = v2.X;
		y2 = v2.Y;

   // Degenerate (H or V line)
		if ((x0 == x1) && (x0 == x2)) {
			LCD_DrawVLine2P( x0, y0, x0, y2);		// vertical line xL = xR
      return;
		}
		uint16_t dy = y2 -y0;
    if (dy == 0) {
      uint16_t xL = x0, xR = x0;
      if (x1 < xL) xL = x1;
      if (x2 < xL) xL = x2;
      if (x1 > xR) xR = x1;
      if (x2 > xR) xR = x2;
			LCD_DrawHLine2P( xL, y0, xR, y0);		// horizontal line, y0=y2; 
      return;
    }
		
		//-- Flat Top triangle 
		if (y0 == y1) {
			if (x0 == x1)  LCD_DrawLine(x0, y0, x2, y2);		// a single general line instead
			else {
				if (x0 >= x1)	flatTriangles(v0, v1, v2);	// x1 <= x0: Left side = Line 12; Right side = Line 02
				else					flatTriangles(v1, v0, v2);	// x1 > x0: Left side = Line 02; Right side = Line 12
			}
			return;
		}
	
   // General -> Flat-bottom + Flat-top
		flatTriangles(v0, v1, v2);
}

static void 	keep_ys_to_Y0(int16_t *rL, int16_t *xL_int, int16_t *xL, uint16_t Y0_ys, int16_t dxL, uint16_t dyL)
{
	uint16_t abs_dxL = ABS(dxL);
	int16_t	inc_xL = (dxL >= 0) ? 1 : -1;	

					*rL = *rL + abs_dxL * (Y0_ys);
					*xL_int += inc_xL * (*rL / dyL);
					*rL = *rL % dyL;
					*xL = *xL_int;	
					if (abs_dxL > dyL) {
						if (*rL != 0){
							if (dxL >= 0)  *xL += 1;
							else					 *xL -= 1;	
						}						
					} 
}

//==================================================================================
static void flatTriangles_inBuffer(Point v0, Point v1, Point v2, sImageBuf *pImageSt)
{ // v1.Y = v2.Y > v0.Y
	uint16_t y0 = v0.Y, y1 = v1.Y, y2 = v2.Y;		// y0 < y1 <= y2 
	uint16_t x0 = v0.X, x1 = v1.X, x2 = v2.X;	// x1 <= x2
	
	//>>>>----------------------------------------------
			uint16_t X0, Y0, width, height;
			uint16_t X_most, Y_most;
			uint16_t *pBuffer;
			uint16_t	Tcolor = DrawProp.TextColor;
		if (pImageSt){  // fill in the image buffer
			DecomposeImStr(&X0, &Y0, &width, &height, &X_most, &Y_most, pImageSt);
			pBuffer = pImageSt->data;
			if ( (y2 < Y0) || (y0 > Y_most) ) return; // out of the buffer range
		} 	
	//<<<<---------------------------------------------

	uint16_t dyL, dyR;
	int16_t dxL, dxR;
	int16_t xL, xR, xL_int, xR_int;
	uint16_t xL_lmt, xR_lmt;
	uint8_t left_Line01;
	uint16_t y, ys = y0, ye;
	xL = xL_int = x0; 
	xR = xR_int = x0;
	dyR = y2 - y0;  // dyR > 0
	dxR = x2 - x0;

	if (y0 == y1) {
			//-- Flat Top triangle 
		// x1 < x0: Left side = Line 12; Right side = Line 02
		flatTop_parameters(&dyL, &dxL, &xL_lmt, x1, y1, x2, y2);
		xR_lmt = xL_lmt;
		xL = xL_int = x1;
		ye = y2;		// flat top: y0 ~ y2, y1 = y0;
	} else {		// if y0 != y1
			//-- Flat Bottom triangle 
		ye = (y2 > y1) ? y1-1 : y1;		// flat bottom: y0 ~ (y1-1); flat top: y1 ~ y2;
			dyL = y1 - y0;
			dxL = x1 - x0;
			int32_t L_slope = (dxL << 16) / dyL;
			int32_t R_slope = (dxR << 16) / dyR;
		if (L_slope == R_slope){
				if (pImageSt) LCD_DrawLine_RGBbuffer(x0, y0, x2, y2, pImageSt);
				else					LCD_DrawLine(x0, y0, x2, y2);		
			return;
		} 		
		if (L_slope > R_slope){
			// Left side = Line 02; Right side = Line 01 
			left_Line01 = 0;
			swap_u16(&dyL, &dyR);
			swap_i16(&dxL, &dxR);
			xR_lmt = x1;
			xL_lmt = x2;
		} else {
			// Left side = Line 01; Right side = Line 02 
			left_Line01 = 1;
			xR_lmt = x2;
			xL_lmt = x1;
		}
	}

	uint16_t abs_dxL, abs_dxR;
	int16_t rL, rR, inc_xL, inc_xR;
	parameters_flatTri(dyL, dxL, &abs_dxL, &rL, &inc_xL);
	parameters_flatTri(dyR, dxR, &abs_dxR, &rR, &inc_xR);

		if (pImageSt) {
			if (ye < Y0){
				//-- Flat Top triangle 
				if (left_Line01)	keep_ys_to_Y0(&rR, &xR_int, &xR, ye-ys, dxR, dyR);
				else							keep_ys_to_Y0(&rL, &xL_int, &xL, ye-ys, dxL, dyL);
				goto loop_end;
			} 
		}

uint16_t y_m_Y0_width;		
loop_start:
		if (pImageSt) {
			if (ye > Y_most) ye = Y_most;
			if ((ye >= Y0) && (ys < Y0) ) {
					keep_ys_to_Y0(&rL, &xL_int, &xL, Y0-ys, dxL, dyL);
					keep_ys_to_Y0(&rR, &xR_int, &xR, Y0-ys, dxR, dyR);
					ys = Y0;	
			}
			y_m_Y0_width = (ys - Y0) * width;
		} 	

		for (y=ys; y<=ye; y++)
		{
			int16_t tL, tR;
			tL = r_And_x_int(&rL, &xL_int, dyL, abs_dxL, inc_xL);			
			if (abs_dxL > dyL) {
				// r == 0 ==> dx> 0, xR = x-1;     dx <0, xL = x+1
				// r != 0 ==> dx> 0, xR = x, x++;  dx <0, xL = x, x--;
				if (dxL >= 0) {
						if (rL != 0) tL++;
				} else	{	// (dxL < 0)
						if (rL == 0) xL = tL + 1;
						else				 xL = tL--;
						if (xL < xL_lmt) xL = xL_lmt;
				}
			}

			tR = r_And_x_int(&rR, &xR_int, dyR, abs_dxR, inc_xR);			
			if (abs_dxR > dyR) {
				// r == 0 ==> dx> 0, xR = x-1;     dx <0, xL = x+1
				// r != 0 ==> dx> 0, xR = x, x++;  dx <0, xL = x, x--;
				if (dxR >= 0) {
						if (rR == 0) xR = tR - 1;
						else				 xR = tR++;
						if (xR > xR_lmt) xR = xR_lmt;
				} else {		// (dxR < 0)
						if (rR != 0) tR--;
				}
			}
			
	//>>>>----------------------------------------------
			if (pImageSt){	 // fill in the image buffer at row Yp
				uint16_t xEnd, xIni, offset;
				if ( (xR >= X0) && (xL <= X_most)){
					xEnd = xR;
					xIni = xL; 
					// draw only when column >= X0
					if (xEnd > X_most) xEnd = X_most;
					if (xIni < X0)	xIni = X0;
					offset = y_m_Y0_width;		// Yp >= Y0	
					offset += (xIni-X0);			// note, x_tmp >=X0;
					// row y:  xIni.....  xEnd
					for (uint16_t j = xIni; j <= xEnd; j++){	// do only when x1 >= x_initial >= X0  > 0
						pBuffer[offset++] = Tcolor;
					}
				}
				y_m_Y0_width += width;
			}			// END of if (pImageSt)
	//<<<<---------------------------------------------
			else 		LCD_DrawHLine2P( xL, y, xR, y);		// horizontal line
				
			xL = tL; xR = tR;
    }
		if (y > y2) return;
		if (pImageSt){	
			if (y > Y_most) return;
		}

loop_end:	
			//-- Flat Top triangle 
		ys = y1; ye = y2;

		if(left_Line01) {
			// Left side is update as Line 12;
			flatTop_parameters(&dyL, &dxL, &xL_lmt, x1, y1, x2, y2);
			parameters_flatTri(dyL, dxL, &abs_dxL, &rL, &inc_xL);
			xL = xL_int = x1;
		} else {
			// Right side is update as Line 12;
			flatTop_parameters(&dyR, &dxR, &xR_lmt, x1, y1, x2, y2);
			parameters_flatTri(dyR, dxR, &abs_dxR, &rR, &inc_xR);
			xR = xR_int = x1;
		}
		goto loop_start;
}

//===================================================================
void LCD_FillTriangle_RGBbuffer3P(pPoint P0, pPoint P1, pPoint P2, sImageBuf *pImageSt)
{
    Point v0 = *P0;
    Point v1 = *P1;
    Point v2 = *P2;

    // Sort by Y s.t. v0.Y <= v1.Y <= v2.Y
    if (v0.Y > v1.Y) Swap(&v0, &v1);
    if (v1.Y > v2.Y) Swap(&v1, &v2);	// ==> v2.Y is the largest
    if (v0.Y > v1.Y) Swap(&v0, &v1);	// ==> v0.Y is the smallest
	uint16_t	x0, y0, x1, y1, x2, y2;
  		x0 = v0.X;
		y0 = v0.Y;
		x1 = v1.X;
		y1 = v1.Y;
		x2 = v2.X;
		y2 = v2.Y;

   // Degenerate (H or V line)
		if ((x0 == x1) && (x0 == x2)) {
		uint8_t HLine = 0;
			// Vertical Line	(x0 = x1 = x2)
			if (pImageSt)	draw_HVline_inBuffer(HLine, x0, y0, x0, y2, pImageSt);	// HLine = 0: vertical line: x0 = x1 = x2
			else 					LCD_DrawVLine2P( x0, y0, x0, y2);		// vertical line xL = xR = x0
			return;
		}
		uint16_t dy = y2 -y0;
    if (dy == 0) {
			// Horizontal Line	(y0 = y1 = y2)
			uint8_t HLine = 1;
			uint16_t xL = x0, xR = x0;
      if (x1 < xL) xL = x1;
      if (x2 < xL) xL = x2;
      if (x1 > xR) xR = x1;
      if (x2 > xR) xR = x2;
			if (pImageSt)	draw_HVline_inBuffer(HLine, xL, y0, xR, y0, pImageSt);	// HLine = 1: horizontal line, y0= y1 = y2;
			else 					LCD_DrawHLine2P( xL, y0, xR, y0);		// horizontal line, y0=y2; 
      return;
    }

		//-- Flat Top triangle 
		if (y0 == y1) {
			if (x0 == x1) {	// a single general line instead
				if (pImageSt) LCD_DrawLine_RGBbuffer(x0, y0, x2, y2, pImageSt);
				else					LCD_DrawLine(x0, y0, x2, y2);		
			}
			else {
				if (x0 < x1)	Swap(&v0, &v1);
				// x1 <= x0: Left side = Line 12; Right side = Line 02
				// x1 > x0: Left side = Line 02; Right side = Line 12 ==> Left = new Line 12; Right  = new Line 02
				if (pImageSt)	flatTriangles_inBuffer(v0, v1, v2, pImageSt);	
				else					flatTriangles(v0, v1, v2);	
			}
			return;
		}
	
		   // General -> Flat-bottom + Flat-top
		if (pImageSt)	flatTriangles_inBuffer(v0, v1, v2, pImageSt);
		else					flatTriangles(v0, v1, v2);
}


/* //-----------------------------------------------------------------------
//===================================================================
void LCD_FillTriangle_RGBbuffer3(pPoint P0, pPoint P1, pPoint P2, sImageBuf *pImageSt)
{
	uint16_t	xp[3], yp[3], i, tmp, x02_mid;
	uint16_t	deltaY1, deltaY2, deltaY12;
  int32_t int_tmp, deltaX1, deltaX2;
  uint16_t x, y, x1, y1;
	uint16_t	Tcolor = DrawProp.TextColor;
	
	//>>>>----------------------------------------------
			uint16_t X0, Y0, width, height;
			uint16_t *pBuffer;
			uint16_t X_most, Y_most;
			uint32_t	offset;
		if (pImageSt){  // fill in the image buffer
			X0 = pImageSt->topLeft.X;
			Y0 = pImageSt->topLeft.Y;
			width = pImageSt->width;
			height = pImageSt->height;
			X_most = X0 + width - 1;
			Y_most = Y0 + height - 1;
			pBuffer = pImageSt->data;
		} 	
	//<<<<---------------------------------------------
		if ((P0 == 0) || (P1 == 0) || (P2 == 0)) return;
		xp[0] = P0->X;
		yp[0] = P0->Y;
		xp[1] = P1->X;
		yp[1] = P1->Y;
		xp[2] = P2->X;
		yp[2] = P2->Y;
		for (i=1; i<3; i++)
		{
			if (yp[0] > yp[i]){
					tmp = xp[i];
				xp[i] = xp[0];
				xp[0] = tmp;
					tmp = yp[i];
				yp[i] = yp[0];
				yp[0] = tmp;
			}
		}	// yp[0] is the smallest value
		if (yp[1] > yp[2]){
					tmp = xp[1];
				xp[1] = xp[2];
				xp[2] = tmp;
					tmp = yp[1];
				yp[1] = yp[2];
				yp[2] = tmp;
		}
		//<<<------------------------------------------------

	// NOTE: Now yp[2] >= yp[1] >= yp[0]
	//----------
	// Point 0 with smallest y; 
	// Point 1 at the middle height
	// Point 2 with largest y
	//------------ 
  deltaX2 = (int32_t) xp[2]- xp[0];		// Line 0_2 is a vertical line
  deltaX1 = (int32_t) xp[1]- xp[0];		// Line 0_1	is a vertical line
	x = xp[0];
	y = yp[0];
	if (deltaX1 == 0 && deltaX2 == 0 ){	// a vertical line ONLY
		if (pImageSt == 0){  // no image buffer
			tmp = yp[2] - y;
			LCD_DrawVLine(x, y, tmp+1);	// a vertical line ONLY
		} else 
	//>>>>----------------------------------------------
		{	 // fill in the image buffer
			if ((x < X0) || (x > X_most)) return;
			if (y < Y0) y = Y0;			// initial row
			y1 = yp[2];						// end row
			if (y1 > Y_most) y1 = Y_most;
			offset = (y-Y0) * width + (x-X0);			// note x >=X0;
			for (uint16_t j = y; j <= y1; j++){		// do only when y1 >= y >= Y0
				pBuffer[offset] = Tcolor;
				offset += width;
			}
		}
	//<<<<---------------------------------------------
		return;		// a vertical line ONLY
	}

	//------------ 
	deltaY2 = (int32_t) yp[2]- yp[0];		// note: deltaY2 >= 0
	if (deltaY2 == 0){	// i.e. 0 = deltaY2 >= deltaY1 >= 0: a horizontal line ONLY
			if( xp[2] < xp[0]){ 
				x = xp[2];
				tmp = xp[0] - x;
			} else {
				tmp = xp[2] - x;
			}
		if (pImageSt == 0){  // no image buffer
			LCD_DrawHLine(x, y, tmp+1);	// a horizontal line
		} else 
	//>>>>----------------------------------------------
		{	 // fill in the image buffer
			if ((y < Y0) || (y > Y_most)) return;
			x1 = tmp + x;						// i.e., xp[0] or xp[2]: end column
			if (x < X0) x = X0;			// initial column
			if (x1 > X_most) x1 = X_most;
			offset = (y-Y0) * width + (x-X0);			// note x >=X0;
			for (uint16_t j = x; j <= x1; j++){		// do only when x1 >= x >= X0
				pBuffer[offset++] = Tcolor;
			}
		}
	//<<<<---------------------------------------------
			return;		// a horizontal line ONLY
	}
	
	// ---------- deltaY2 (yp[2]- yp[0]) > 0 in the following --------------------
//	x = xp[0];		// has been set earlier
//	y = yp[0];		// has been set earlier
	deltaY1 = (int32_t) yp[1]- yp[0];		// always deltaY1 >= 0
	deltaY12 = (int32_t) yp[2]- yp[1];		// Line 1_2
	// Note: For P2 == 0, deltaY12 = 0 definitely, since yp[2] = yp[1] and xp[2]- xp[1].
	
	if (deltaY1 == 0){
	// If deltaY1 == 0 (yp[1] = yp[0]), there is NO the 1st triangle; 
			x02_mid = xp[0];		// (x02_mid, yp[1]) middle point of Line 0_2 is set at point 0
									// jump to the 2nd triangle.
	} else {
		// get the middle point of Line 0_2, which at the same Y level (yp[1]) of point 1
		// i.e., get (x02_mid, yp[1])
		if (deltaY12 == 0 ){
			x02_mid = xp[2];	// middle point of Line 0_2 is set at point 2; (x02_mid, yp[1]) = (xp[2], yp[2])
		} else{
			int_tmp = (deltaX2 * deltaY1) /deltaY2;
			x02_mid = (uint16_t)(xp[0] + int_tmp); // xp[0] + (yp[1]-yp[0])*((xp[2]-xp[0])/(yp[1]-yp[0]))
		}
	//====== 1st subtriangle (P0-P1-Px, where Px=(x02_mid, yp[1]) )
		if ( x02_mid < xp[1] ){ 
			// if Line 0_2 is left to Line 0_1, exchange THEM.
			int_tmp = deltaX1;
			deltaX1 = deltaX2;				// original Line 0_2
			deltaX2 = int_tmp;				// original Line 0_1
			int_tmp = deltaY1;
			deltaY1 = deltaY2;
			deltaY2 = int_tmp;
		}

			y1 = yp[1];	// end row
			i = 0;			// i = y - xp[0]
			if (pImageSt)
	//>>>>----------------------------------------------
			{	 // fill in the image buffer
				if ( y < Y0){			// note, y = xp[0]
					i = Y0 - y;			// i = Y0 - xp[0]
					y = Y0;					// initial row
				}
				if (y1 > Y_most) y1 = Y_most;		// end row
			}
	//<<<<---------------------------------------------
		for (uint16_t Yp=y; Yp<=y1; Yp++)	// downward
		{
				int32_t x_tmp, x_initial, length;
			
			if	(i == 0) {			// i = y - xp[0]
				x_tmp = 0;
				length = 1;
			} else {
				x_tmp = deltaX1*i /deltaY1;
				length = deltaX2*i /deltaY2- x_tmp + 1; // Note: length must >= 1, since new Line 0_2 is right to new Line 0_1
			}
			x_initial = x + x_tmp;	// x_initial
			
			if (pImageSt == 0){  // no image buffer
				if (length == 1) LCD_DrawPixel((uint16_t) x_initial, Yp, Tcolor); // point (x_initial, Yp)
				else LCD_DrawHLine((uint16_t) x_initial, Yp, (uint16_t) length);
			} else 
	//>>>>----------------------------------------------
			{	 // fill in the image buffer at row Yp
				x1 = x_initial + length - 1;		// x_end
				if ( x1 >= X0){								// draw only when column >= X0
					if (x1 > X_most) x1 = X_most;
					if (x_initial < X0)	x_initial = X0;
					offset = (Yp - Y0) * width;		// Yp >= Y0	
					offset += (x_initial-X0);			// note, x_tmp >=X0;
					// row Yp:  x_initial .....  x1
					for (uint16_t j = x_initial; j <= x1; j++){	// do only when x1 >= x_initial >= X0  > 0
						pBuffer[offset++] = Tcolor;
					}
				}
			}
	//<<<<---------------------------------------------
			i++;
		} 
		if (deltaY12 == 0 ) return; // There is NO the 2nd triangle.
	}			// else END of if (deltaY1 == 0) 
	//====== 2nd subtriangle 
second_Tri:	
	if ( x02_mid < xp[1] ){ // Original Line 0_2 is left to Line 1_2
		deltaX2 = (int32_t) xp[1]- xp[2];		// line 2_1
		deltaY2 = (int32_t) yp[2]- yp[1];		
		deltaX1 = (int32_t) xp[0]- xp[2];		// left side delta: line 2_0 on the left side
		deltaY1 = (int32_t) yp[2]- yp[0];	
	} else{	// for x0_mid >= xp[1]; Original Line 0_2 is right to Line 1_2
		deltaX1 = (int32_t) xp[1]- xp[2];		// left side delta: line 2_1 on the left side
		deltaY1 = (int32_t) yp[2]- yp[1];		
		deltaX2 = (int32_t) xp[0]- xp[2];		// line 2_0
		deltaY2 = (int32_t) yp[2]- yp[0];		
	}
		x = xp[2];
		y = yp[1];		// initial row
		y1 = yp[2];		// end row
		i = 0;				// i = yp[2] - L, first L = y1
		if (pImageSt)
	//>>>>----------------------------------------------
		{	 // fill in the image buffer
				if ( y < Y0) y = Y0;	// initial row
				if (y1 > Y_most){			// end row
					i = y1 - Y_most;		// i = yp[2] - L, first L = Y_most
					y1 = Y_most;				// end row
				}
		}
	//<<<<---------------------------------------------
		for (int16_t L=y1; L>=y; L--)			// upward, NOTE: yp[1] may be 0
		{
				int32_t x_tmp, length;
				
			if	(i == 0) {					// i = yp[2] - L
				x_tmp = 0;
				length = 1;
			} else {
				x_tmp = deltaX1*i / deltaY1;
				length = deltaX2*i /deltaY2- x_tmp + 1; // Note: , length must >= 1, since (deltaX2/deltaY2) >= (deltaX1/deltaY1) > 0
			}

			if (pImageSt == 0){  // no image buffer
				if (length == 1) LCD_DrawPixel((uint16_t) (x+x_tmp), L, Tcolor);
				else LCD_DrawHLine((uint16_t) (x+x_tmp), L, (uint16_t) length);
			} else 
	//>>>>----------------------------------------------
			{	 // fill in the image buffer
				x_tmp = x + x_tmp;	// x_initial >= 0
				x1 = x_tmp + length - 1;		// note, x_end >= x_initial
					if (x1 >= X0){				// draw only when column >= X0
						if (x1 > X_most) x1 = X_most;
						if (x_tmp < X0)	x_tmp = X0;
						offset = (L-Y0) * width;			// 
						offset += (x_tmp-X0);			// note, x_tmp >=X0;
						for (uint16_t j = x_tmp; j <= x1; j++){	// do only when x1 >= x_tmp >= X0 > 0
							pBuffer[offset++] = Tcolor;
						}
					}
			}
	//<<<<---------------------------------------------
			i++;
		}	
}
*/
