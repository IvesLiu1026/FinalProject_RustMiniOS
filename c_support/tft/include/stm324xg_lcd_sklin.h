/**
  ******************************************************************************
  * @file    stm324xg_eval_lcd.h
  * @author  MCD Application Team
  * @version V1.0.0
  * @date    30-September-2011
  * @brief   This file contains all the functions prototypes for the stm324xg_eval_lcd.c
  *          driver.
  ******************************************************************************
  * @attention
  *
  * THE PRESENT FIRMWARE WHICH IS FOR GUIDANCE ONLY AIMS AT PROVIDING CUSTOMERS
  * WITH CODING INFORMATION REGARDING THEIR PRODUCTS IN ORDER FOR THEM TO SAVE
  * TIME. AS A RESULT, STMICROELECTRONICS SHALL NOT BE HELD LIABLE FOR ANY
  * DIRECT, INDIRECT OR CONSEQUENTIAL DAMAGES WITH RESPECT TO ANY CLAIMS ARISING
  * FROM THE CONTENT OF SUCH FIRMWARE AND/OR THE USE MADE BY CUSTOMERS OF THE
  * CODING INFORMATION CONTAINED HEREIN IN CONNECTION WITH THEIR PRODUCTS.
  *
  * <h2><center>&copy; COPYRIGHT 2011 STMicroelectronics</center></h2>
  ******************************************************************************  
  */ 

/* Define to prevent recursive inclusion -------------------------------------*/
#ifndef __STM324XG_EVAL_LCD_H
#define __STM324XG_EVAL_LCD_H

#ifdef __cplusplus
 extern "C" {
#endif 

#include "stm32f407xx.h"
#include <stdint.h>

uint8_t LCD_DrawJPG(uint16_t Xpos, uint16_t Ypos, uint8_t *p_jpgAdd, uint16_t Width, uint16_t Height);
// if one of Width and Height is 0, use the original size of the image file
char* get_JPG_error_code(void);		// for JPG error message text
uint8_t LCD_DrawGIF(uint16_t Xpos, uint16_t Ypos, uint8_t *p_gifAddress, uint16_t S_Width, uint16_t S_Height);	

/** 
  * @brief  LCD Registers  
   HAL (e.g., at ili9325.h)
  */ 

#define LCD_REG_0             0x00
#define LCD_REG_1             0x01
#define LCD_REG_2             0x02
#define LCD_REG_3             0x03
#define LCD_REG_4             0x04
#define LCD_REG_5             0x05
#define LCD_REG_6             0x06
#define LCD_REG_7             0x07
#define LCD_REG_8             0x08
#define LCD_REG_9             0x09
#define LCD_REG_10            0x0A
#define LCD_REG_12            0x0C
#define LCD_REG_13            0x0D
#define LCD_REG_14            0x0E
#define LCD_REG_15            0x0F
#define LCD_REG_16            0x10
#define LCD_REG_17            0x11
#define LCD_REG_18            0x12
#define LCD_REG_19            0x13
#define LCD_REG_20            0x14
#define LCD_REG_21            0x15
#define LCD_REG_22            0x16
#define LCD_REG_23            0x17
#define LCD_REG_24            0x18
#define LCD_REG_25            0x19
#define LCD_REG_26            0x1A
#define LCD_REG_27            0x1B
#define LCD_REG_28            0x1C
#define LCD_REG_29            0x1D
#define LCD_REG_30            0x1E
#define LCD_REG_31            0x1F
#define LCD_REG_32            0x20
#define LCD_REG_33            0x21
#define LCD_REG_34            0x22
#define LCD_REG_36            0x24
#define LCD_REG_37            0x25
#define LCD_REG_40            0x28
#define LCD_REG_41            0x29
#define LCD_REG_43            0x2B
#define LCD_REG_45            0x2D
#define LCD_REG_48            0x30
#define LCD_REG_49            0x31
#define LCD_REG_50            0x32
#define LCD_REG_51            0x33
#define LCD_REG_52            0x34
#define LCD_REG_53            0x35
#define LCD_REG_54            0x36
#define LCD_REG_55            0x37
#define LCD_REG_56            0x38
#define LCD_REG_57            0x39
#define LCD_REG_58            0x3A
#define LCD_REG_59            0x3B
#define LCD_REG_60            0x3C
#define LCD_REG_61            0x3D
#define LCD_REG_62            0x3E
#define LCD_REG_63            0x3F
#define LCD_REG_64            0x40
#define LCD_REG_65            0x41
#define LCD_REG_66            0x42
#define LCD_REG_67            0x43
#define LCD_REG_68            0x44
#define LCD_REG_69            0x45
#define LCD_REG_70            0x46
#define LCD_REG_71            0x47
#define LCD_REG_72            0x48
#define LCD_REG_73            0x49
#define LCD_REG_74            0x4A
#define LCD_REG_75            0x4B
#define LCD_REG_76            0x4C
#define LCD_REG_77            0x4D
#define LCD_REG_78            0x4E
#define LCD_REG_79            0x4F
#define LCD_REG_80            0x50
#define LCD_REG_81            0x51
#define LCD_REG_82            0x52
#define LCD_REG_83            0x53
#define LCD_REG_96            0x60
#define LCD_REG_97            0x61
#define LCD_REG_106           0x6A
#define LCD_REG_118           0x76
#define LCD_REG_128           0x80
#define LCD_REG_129           0x81
#define LCD_REG_130           0x82
#define LCD_REG_131           0x83
#define LCD_REG_132           0x84
#define LCD_REG_133           0x85
#define LCD_REG_134           0x86
#define LCD_REG_135           0x87
#define LCD_REG_136           0x88
#define LCD_REG_137           0x89
#define LCD_REG_139           0x8B
#define LCD_REG_140           0x8C
#define LCD_REG_141           0x8D
#define LCD_REG_143           0x8F
#define LCD_REG_144           0x90
#define LCD_REG_145           0x91
#define LCD_REG_146           0x92
#define LCD_REG_147           0x93
#define LCD_REG_148           0x94
#define LCD_REG_149           0x95
#define LCD_REG_150           0x96
#define LCD_REG_151           0x97
#define LCD_REG_152           0x98
#define LCD_REG_153           0x99
#define LCD_REG_154           0x9A
#define LCD_REG_157           0x9D
#define LCD_REG_192           0xC0
#define LCD_REG_193           0xC1
#define LCD_REG_229           0xE5


//###############################
/** @defgroup FONTS_Exported_Types
  * @{
  */ 
#ifndef _sFONT
#define _sFONT
typedef struct 	// also deined in .\Fonts\fonts.h
{    
  const uint8_t *table;
  uint16_t Width;
  uint16_t Height;
} sFONT;

/** @defgroup FONTS_Exported_Constants
  * @{
  */ 
#define LCD_LINE(x) ((x) * (((sFONT *)LCD_GetFont())->Height))

#include "Fonts/fonts.h"
#endif


typedef struct 
{
  uint16_t X;
  uint16_t Y;
}Point, * pPoint; 

typedef struct {    
//  uint16_t x0;			// topleft point
//  uint16_t y0;			// topleft point
	Point		topLeft;
  uint16_t width;
  uint16_t height;
	uint16_t *data;
} sImageBuf;


/** 
  * @brief  Line mode structures definition
  */ 
typedef enum
{
  LEFT_MODE               = 0x00,     /*!< Left mode   */
  CENTER_MODE             = 0x01,    /*!< Center mode */
  RIGHT_MODE              = 0x02,    /*!< Right mode  */
}Line_ModeTypdef;

/** 
  * @brief  LCD color  
  */ 
#define LCD_COLOR_BLUE          0x001F
#define LCD_COLOR_GREEN         0x07E0
#define LCD_COLOR_RED           0xF800
#define LCD_COLOR_CYAN          0x07FF
#define LCD_COLOR_MAGENTA       0xF81F
#define LCD_COLOR_YELLOW        0xFFE0
#define LCD_COLOR_LIGHTBLUE     0x841F
#define LCD_COLOR_LIGHTGREEN    0x87F0
#define LCD_COLOR_LIGHTRED      0xFC10
#define LCD_COLOR_LIGHTCYAN     0x87FF
#define LCD_COLOR_LIGHTMAGENTA  0xFC1F
#define LCD_COLOR_LIGHTYELLOW   0xFFF0
#define LCD_COLOR_DARKBLUE      0x0010
#define LCD_COLOR_DARKGREEN     0x0400
#define LCD_COLOR_DARKRED       0x8000
#define LCD_COLOR_DARKCYAN      0x0410
#define LCD_COLOR_DARKMAGENTA   0x8010
#define LCD_COLOR_DARKYELLOW    0x8400
#define LCD_COLOR_WHITE         0xFFFF
#define LCD_COLOR_LIGHTGRAY     0xD69A
#define LCD_COLOR_GRAY          0x8410
#define LCD_COLOR_DARKGRAY      0x4208
#define LCD_COLOR_BLACK         0x0000
#define LCD_COLOR_BROWN         0xA145
#define LCD_COLOR_ORANGE        0xFD20

//--- addittion by S.K. Lin ---------
#define LCD_COLOR_GREY           0xF7DE
#define LCD_COLOR_BLUE2          0x051F
#define LCD_COLOR_CYAN2    	 0x7FFF


 


/** @defgroup STM324xG_EVAL_LCD_Exported_Macros
  * @{
  */
#define ASSEMBLE_RGB(R ,G, B)    ((((R)& 0xF8) << 8) | (((G) & 0xFC) << 3) | (((B) & 0xF8) >> 3)) 
/**
  * @}
  */ 

//#define     __O     volatile             /*!< Defines 'write only' permissions                */
//#define     __IO    volatile             /*!< Defines 'read / write' permissions              */
void LCD_Init(void);
/** @defgroup STM324xG_EVAL_LCD_Exported_Functions
  * @{
  */ 

//<<<<-----------------

void LCD_SetColors(uint16_t _TextColor, uint16_t _BackColor); 
void LCD_GetColors(uint16_t *_TextColor, uint16_t *_BackColor);

void LCD_DrawMonoPict(const uint32_t *Pict);
void LCD_PolyLineClosed(pPoint p, uint16_t pCount, uint8_t Closed);
#define LCD_PolyLine(Points, PointCount) LCD_PolyLineClosed(Points, PointCount, 0)
#define LCD_ClosedPolyLine(Points, PointCount) LCD_PolyLineClosed(Points, PointCount, 1)
void LCD_PolyLineRelative(pPoint Points, uint16_t PointCount);
void LCD_ClosedPolyLineRelative(pPoint Points, uint16_t PointCount);

void ReverseLCD(void);
void NormalLCD(void);



// The following items are used (but aliased with "BSP_" as header) in stm324xg_eval_lcd.c,  V2.0.1 on 26-February-2014 
	void LCD_SetTextColor(__IO uint16_t Color);
	void LCD_SetBackColor(__IO uint16_t Color);
	void LCD_SetFont(sFONT *fonts);
	sFONT *LCD_GetFont(void);
	
	void LCD_Clear(uint16_t Color);
	void LCD_ClearStringLine(uint16_t LineNr);
	void LCD_DisplayStringAt(uint16_t Xpos, uint16_t Ypos, char* Text, Line_ModeTypdef Mode);
	void LCD_DisplayStringLineCol(uint8_t LineNr, uint16_t ColNr, char *ptr);
  #define LCD_DisplayStringAtLine(LineNr, ptr)   LCD_DisplayStringLineCol(LineNr, 0, ptr)

	uint16_t LCD_ReadPixel(uint16_t Xpos, uint16_t Ypos);
	void LCD_DrawPixel(uint16_t Xpos, uint16_t Ypos, uint16_t RGB_Code);
	void LCD_DrawHLine(uint16_t Xpos, uint16_t Ypos, uint16_t Length);
	void LCD_DrawVLine(uint16_t Xpos, uint16_t Ypos, uint16_t Length);
	void LCD_DrawHV2P(uint8_t HLine, uint16_t x0, uint16_t y0, uint16_t x1, uint16_t y1);
	void LCD_DrawSimpleCross(uint16_t Xpos, uint16_t Ypos, uint16_t Length);
	void LCD_DrawLine(uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2);
#define LCD_DrawLine2P(x1, y1, x2, y2)	 LCD_DrawLine(x1, y1, x2, y2)
	void LCD_DrawRect(uint16_t Xpos, uint16_t Ypos, uint16_t Width, uint16_t Height);
	void LCD_DrawCircle(uint16_t Xpos, uint16_t Ypos, uint16_t Radius);
#define LCD_DrawPolygon(Points, PointCount)			LCD_PolyLineClosed(Points, PointCount, 1)
	void LCD_DrawBitmap(uint16_t Xpos, uint16_t Ypos, uint8_t *BmpAddress);
	void LCD_DrawRGBImage(uint16_t Xpos, uint16_t Ypos, uint16_t Width, uint16_t Height, uint8_t *pbmp);
	void LCD_FillRect(uint16_t Xpos, uint16_t Ypos, uint16_t Width, uint16_t Height);
	void LCD_FillCircle(uint16_t Xpos, uint16_t Ypos, uint16_t Radius);
	void LCD_FillPolygon(pPoint Points, uint16_t PointCount);
	void LCD_FillTriangle(pPoint Points);
	void LCD_DrawEllipse(int Xpos, int Ypos, int XRadius, int YRadius);
	void LCD_FillEllipse(int Xpos, int Ypos, int XRadius, int YRadius);

	void LCD_DisplayOff(void);
	void LCD_DisplayOn(void);

//============== NEW by Shir-Kuan Lin ++++++++++++
void LCD_DrawRect_pt0_pt1(uint16_t Xul, uint16_t Yul, uint16_t Xdr, uint16_t Ydr);
void LCD_DrawRect_by_2Points(Point *pA);
void LCD_ReadRGBImage(uint16_t Xpos, uint16_t Ypos, uint16_t Xsize, uint16_t Ysize, uint8_t *pdata);
void LCD_DrawRGB_Transparent(uint16_t Xpos, uint16_t Ypos, uint16_t Xsize, uint16_t Ysize, uint8_t *pdata, uint16_t transColor);
void LCD_FillPolygon_RGBbuffer(pPoint Points, uint16_t PointCount, sImageBuf *pImageSt);
void LCD_DrawRGBbuffer_Transparent(sImageBuf *pImageSt, uint16_t transColor);
void LCD_DrawRGBbuffer(sImageBuf *pImageSt);
void LCD_DrawRGBbuffer_skipRect(sImageBuf *imageSt, Point *skip_p);
void LCD_FillRectBack_skipRect(Point *DrawB_p, Point *skip_p);
void LCD_SaveColors(void);
void LCD_RestoreColors(void);
void LCD_SaveFont(void);
void LCD_RestoreFont(void);
void LCD_RGB_Test(void);
void MenuInit(void);
uint16_t	LCD_GetTextColor(void);
uint16_t	LCD_GetBackColor(void);
void LCD_digital(uint8_t row_no, uint8_t col_no, int16_t val, char *ptr);
// Example for no ptr: LCD_digital(row_no, col_no, val,  (char*)'\n');



/** 
  * @brief  LCD color  
  */ 
#define BLUE          0x001F
#define GREEN         0x07E0
#define RED           0xF800
#define CYAN          0x07FF
#define MAGENTA       0xF81F
#define YELLOW        0xFFE0
#define LIGHTBLUE     0x841F
#define LIGHTGREEN    0x87F0
#define LIGHTRED      0xFC10
#define LIGHTCYAN     0x87FF
#define LIGHTMAGENTA  0xFC1F
#define LIGHTYELLOW   0xFFF0
#define DARKBLUE      0x0010
#define DARKGREEN     0x0400
#define DARKRED       0x8000
#define DARKCYAN      0x0410
#define DARKMAGENTA   0x8010
#define DARKYELLOW    0x8400
#define WHITE         0xFFFF
#define LIGHTGRAY     0xD69A
#define GRAY          0x8410
#define DARKGRAY      0x4208
#define BLACK         0x0000
#define BROWN         0xA145
#define ORANGE        0xFD20

//--- addittion by S.K. Lin ---------
#define GREY          0xF7DE
#define BLUE2         0x051F
#define CYAN2    	  	0x7FFF
#define MIDDLEYELLOW        0xEFB0


#ifdef __cplusplus
}
#endif


/*****END OF FILE****/

//===========================================
/** @defgroup STM324xG_EVAL_LCD_Exported_Functions
  * @{
  */ 
uint16_t LCD_Pixel_Width(void);
uint16_t LCD_Pixel_Height(void);

#define BSP_LCD_GetXSize()	LCD_Pixel_Width()
#define BSP_LCD_GetYSize()	LCD_Pixel_Height()
#define  BSP_LCD_GetTextColor()  (DrawProp.TextColor) 
#define  BSP_LCD_GetBackColor()  (DrawProp.BackColor) 


#define  BSP_LCD_Init						TaoTao_LCD_Init
#define  BSP_LCD_SetTextColor		LCD_SetTextColor
#define  BSP_LCD_SetBackColor		LCD_SetBackColor
#define  BSP_LCD_SetFont				LCD_SetFont
#define  BSP_LCD_GetFont				LCD_GetFont

#define  BSP_LCD_Clear					LCD_Clear
#define  BSP_LCD_ClearStringLine			LCD_ClearStringLine
#define  BSP_LCD_DisplayStringAtLine	LCD_DisplayStringAtLine
#define  BSP_LCD_DisplayStringAt			BSP_LCD_DisplayStringAt
#define  BSP_LCD_DisplayChar					LCD_DisplayChar

#define  BSP_LCD_ReadPixel						LCD_ReadPixel
#define  BSP_LCD_DrawPixel						LCD_DrawPixel
#define  BSP_LCD_DrawHLine						LCD_DrawHLine
#define  BSP_LCD_DrawVLine						LCD_DrawVLine
#define  BSP_LCD_DrawLine							LCD_DrawLine
#define  BSP_LCD_DrawRect							LCD_DrawRect
#define  BSP_LCD_DrawCircle						LCD_DrawCircle
#define  BSP_LCD_DrawPolygon					LCD_DrawPolygon
#define  BSP_LCD_DrawEllipse					LCD_DrawEllipse
#define  BSP_LCD_DrawBitmap						LCD_DrawBitmap
#define  BSP_LCD_DrawRGBImage					LCD_DrawRGBImage
#define  BSP_LCD_FillRect							LCD_FillRect
#define  BSP_LCD_FillCircle						LCD_FillCircle
#define  BSP_LCD_FillPolygon					LCD_FillPolygon
#define  BSP_LCD_FillEllipse					LCD_FillEllipse

#define  BSP_LCD_DisplayOff						LCD_DisplayOff
#define  BSP_LCD_DisplayOn						LCD_DisplayOn



#endif /* __STM324xG_EVAL_LCD_H */

