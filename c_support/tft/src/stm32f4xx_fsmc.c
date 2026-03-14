/**
  ******************************************************************************
  * @file    stm32f4xx_fsmc.c
  * @author  MCD Application Team
  * @brief   FSMC Low Layer HAL module driver.
  *    
  *          This file provides firmware functions to manage the following 
  *          functionalities of the Flexible Static Memory Controller (FSMC) peripheral memories:
  *           + Initialization/de-initialization functions
  *           + Peripheral Control functions 
  *           + Peripheral State functions
  *         
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

/* Includes ------------------------------------------------------------------*/
#include "stm32f4xx_fsmc.h"
#ifndef assert_param
  #define assert_param(expr) ((void)0)
#endif

/** @addtogroup STM32F4xx_StdPeriph_Driver
  * @{
  */

/** @defgroup FSMC_LL compatiable 
  * @brief FSMC driver modules
  * @{
  */ 

	
/**
  * @brief  Initialize the FSMC_NORSRAM device according to the specified
  *         control parameters in the FSMC_NORSRAM_InitTypeDef
  * @param  Device Pointer to NORSRAM device instance
  * @param  Init Pointer to NORSRAM Initialization structure   
  * @retval HAL status
  */
void  FSMC_NORSRAM_Init(FSMC_Bank1_TypeDef *Device, FSMC_NORSRAM_InitTypeDef* Init)
{ 
  uint32_t tmpr = 0U;
    
  /* Check the parameters */
  assert_param(IS_FSMC_NORSRAM_DEVICE(Device));
  assert_param(IS_FSMC_NORSRAM_BANK(Init->NSBank));
  assert_param(IS_FSMC_MUX(Init->DataAddressMux));
  assert_param(IS_FSMC_MEMORY(Init->MemoryType));
  assert_param(IS_FSMC_NORSRAM_MEMORY_WIDTH(Init->MemoryDataWidth));
  assert_param(IS_FSMC_BURSTMODE(Init->BurstAccessMode));
  assert_param(IS_FSMC_WAIT_POLARITY(Init->WaitSignalPolarity));
#if defined(STM32F405xx) || defined(STM32F415xx) || defined(STM32F407xx) || defined(STM32F417xx)
  assert_param(IS_FSMC_WRAP_MODE(Init->WrapMode));
#endif /* STM32F405xx || STM32F415xx || STM32F407xx || STM32F417xx */
  assert_param(IS_FSMC_WAIT_SIGNAL_ACTIVE(Init->WaitSignalActive));
  assert_param(IS_FSMC_WRITE_OPERATION(Init->WriteOperation));
  assert_param(IS_FSMC_WAITE_SIGNAL(Init->WaitSignal));
  assert_param(IS_FSMC_EXTENDED_MODE(Init->ExtendedMode));
  assert_param(IS_FSMC_ASYNWAIT(Init->AsynchronousWait));
  assert_param(IS_FSMC_WRITE_BURST(Init->WriteBurst));
  assert_param(IS_FSMC_PAGESIZE(Init->PageSize));
#if defined(STM32F412Zx) || defined(STM32F412Vx) || defined(STM32F412Rx) || defined(STM32F413xx) || defined(STM32F423xx)
  assert_param(IS_FSMC_WRITE_FIFO(Init->WriteFifo));
  assert_param(IS_FSMC_CONTINOUS_CLOCK(Init->ContinuousClock));
#endif /* STM32F412Zx || STM32F412Vx || STM32F413xx || STM32F423xx */
  
  /* Get the BTCR register value */
  tmpr = Device->BTCR[Init->NSBank];

#if defined(STM32F405xx) || defined(STM32F415xx) || defined(STM32F407xx) || defined(STM32F417xx)
  /* Clear MBKEN, MUXEN, MTYP, MWID, FACCEN, BURSTEN, WAITPOL, WRAPMOD, WAITCFG, WREN,
           WAITEN, EXTMOD, ASYNCWAIT, CPSIZE and CBURSTRW bits */
  tmpr &= ((uint32_t)~(FSMC_BCR1_MBKEN     | FSMC_BCR1_MUXEN    | FSMC_BCR1_MTYP     | \
                       FSMC_BCR1_MWID      | FSMC_BCR1_FACCEN   | FSMC_BCR1_BURSTEN  | \
                       FSMC_BCR1_WAITPOL   | FSMC_BCR1_WRAPMOD  | FSMC_BCR1_WAITCFG  | \
                       FSMC_BCR1_WREN      | FSMC_BCR1_WAITEN   | FSMC_BCR1_EXTMOD   | \
                       FSMC_BCR1_ASYNCWAIT | FSMC_BCR1_CPSIZE   | FSMC_BCR1_CBURSTRW));
  /* Set NORSRAM device control parameters */
  tmpr |= (uint32_t)(Init->DataAddressMux       |\
                     Init->MemoryType           |\
                     Init->MemoryDataWidth      |\
                     Init->BurstAccessMode      |\
                     Init->WaitSignalPolarity   |\
                     Init->WrapMode             |\
                     Init->WaitSignalActive     |\
                     Init->WriteOperation       |\
                     Init->WaitSignal           |\
                     Init->ExtendedMode         |\
                     Init->AsynchronousWait     |\
                     Init->PageSize             |\
                     Init->WriteBurst
                     );
#else /* STM32F412Zx || STM32F412Vx || STM32F412Rx || STM32F413xx || STM32F423xx */
  /* Clear MBKEN, MUXEN, MTYP, MWID, FACCEN, BURSTEN, WAITPOL, WAITCFG, WREN,
           WAITEN, EXTMOD, ASYNCWAIT,CPSIZE,  CBURSTRW, CCLKEN and WFDIS bits */
  tmpr &= ((uint32_t)~(FSMC_BCR1_MBKEN     | FSMC_BCR1_MUXEN    | FSMC_BCR1_MTYP      | \
                       FSMC_BCR1_MWID      | FSMC_BCR1_FACCEN   | FSMC_BCR1_BURSTEN   | \
                       FSMC_BCR1_WAITPOL   | FSMC_BCR1_WAITCFG  | FSMC_BCR1_WREN      | \
                       FSMC_BCR1_WAITEN    | FSMC_BCR1_EXTMOD   | FSMC_BCR1_ASYNCWAIT | \
                       FSMC_BCR1_CPSIZE    | FSMC_BCR1_CBURSTRW | FSMC_BCR1_CCLKEN    | \
                       FSMC_BCR1_WFDIS));
  /* Set NORSRAM device control parameters */
  tmpr |= (uint32_t)(Init->DataAddressMux       |\
                     Init->MemoryType           |\
                     Init->MemoryDataWidth      |\
                     Init->BurstAccessMode      |\
                     Init->WaitSignalPolarity   |\
                     Init->WaitSignalActive     |\
                     Init->WriteOperation       |\
                     Init->WaitSignal           |\
                     Init->ExtendedMode         |\
                     Init->AsynchronousWait     |\
                     Init->WriteBurst           |\
                     Init->ContinuousClock      |\
                     Init->PageSize             |\
                     Init->WriteFifo);
#endif /* STM32F405xx || STM32F415xx || STM32F407xx || STM32F417xx */ 
            
  if(Init->MemoryType == FSMC_MEMORY_TYPE_NOR)
  {
    tmpr |= (uint32_t)FSMC_NORSRAM_FLASH_ACCESS_ENABLE;
  }

  Device->BTCR[Init->NSBank] = tmpr;

#if defined(STM32F412Zx) || defined(STM32F412Vx) || defined(STM32F412Rx) || defined(STM32F413xx) || defined(STM32F423xx)
  /* Configure synchronous mode when Continuous clock is enabled for bank2..4 */
  if((Init->ContinuousClock == FSMC_CONTINUOUS_CLOCK_SYNC_ASYNC) && (Init->NSBank != FSMC_NORSRAM_BANK1))
  {
    Device->BTCR[FSMC_NORSRAM_BANK1] |= (uint32_t)(Init->ContinuousClock);
  }

  if(Init->NSBank != FSMC_NORSRAM_BANK1)
  {
    Device->BTCR[FSMC_NORSRAM_BANK1] |= (uint32_t)(Init->WriteFifo);
  }
#endif /* STM32F412Zx || STM32F412Vx || STM32F412Rx || STM32F413xx || STM32F423xx */

}

/**
  * @brief  DeInitialize the FSMC_NORSRAM peripheral 
  * @param  Device Pointer to NORSRAM device instance
  * @param  ExDevice Pointer to NORSRAM extended mode device instance  
  * @param  Bank NORSRAM bank number  
  * @retval HAL status
  */
void FSMC_NORSRAM_DeInit(FSMC_NORSRAM_TypeDef *Device, FSMC_NORSRAM_EXTENDED_TypeDef *ExDevice, uint32_t Bank)
{
  /* Check the parameters */
  assert_param(IS_FSMC_NORSRAM_DEVICE(Device));
  assert_param(IS_FSMC_NORSRAM_EXTENDED_DEVICE(ExDevice));
  assert_param(IS_FSMC_NORSRAM_BANK(Bank));

  /* Disable the FSMC_NORSRAM device */
  __FSMC_NORSRAM_DISABLE(Device, Bank);
  
  /* De-initialize the FSMC_NORSRAM device */
  /* FSMC_NORSRAM_BANK1 */
  if(Bank == FSMC_NORSRAM_BANK1)
  {
    Device->BTCR[Bank] = 0x000030DBU;    
  }
  /* FSMC_NORSRAM_BANK2, FSMC_NORSRAM_BANK3 or FSMC_NORSRAM_BANK4 */
  else
  {   
    Device->BTCR[Bank] = 0x000030D2U; 
  }
  
  Device->BTCR[Bank + 1U] = 0x0FFFFFFFU;
  ExDevice->BWTR[Bank]    = 0x0FFFFFFFU;
   
}


/**
  * @brief  Initialize the FSMC_NORSRAM Timing according to the specified
  *         parameters in the FSMC_NORSRAM_TimingTypeDef
  * @param  Device Pointer to NORSRAM device instance
  * @param  Timing Pointer to NORSRAM Timing structure
  * @param  Bank NORSRAM bank number  
  * @retval HAL status
  */
void FSMC_NORSRAM_Timing_Init(FSMC_NORSRAM_TypeDef *Device, FSMC_NORSRAM_TimingTypeDef *Timing, uint32_t Bank)
{
  uint32_t tmpr = 0U;
  
  /* Check the parameters */
  assert_param(IS_FSMC_NORSRAM_DEVICE(Device));
  assert_param(IS_FSMC_ADDRESS_SETUP_TIME(Timing->AddressSetupTime));
  assert_param(IS_FSMC_ADDRESS_HOLD_TIME(Timing->AddressHoldTime));
  assert_param(IS_FSMC_DATASETUP_TIME(Timing->DataSetupTime));
  assert_param(IS_FSMC_TURNAROUND_TIME(Timing->BusTurnAroundDuration));
  assert_param(IS_FSMC_CLK_DIV(Timing->CLKDivision));
  assert_param(IS_FSMC_DATA_LATENCY(Timing->DataLatency));
  assert_param(IS_FSMC_ACCESS_MODE(Timing->AccessMode));
  assert_param(IS_FSMC_NORSRAM_BANK(Bank));
  
  /* Get the BTCR register value */
  tmpr = Device->BTCR[Bank + 1U];

  /* Clear ADDSET, ADDHLD, DATAST, BUSTURN, CLKDIV, DATLAT and ACCMOD bits */
  tmpr &= ((uint32_t)~(FSMC_BTR1_ADDSET  | FSMC_BTR1_ADDHLD | FSMC_BTR1_DATAST | \
                       FSMC_BTR1_BUSTURN | FSMC_BTR1_CLKDIV | FSMC_BTR1_DATLAT | \
                       FSMC_BTR1_ACCMOD));
  
  /* Set FSMC_NORSRAM device timing parameters */  
  tmpr |= (uint32_t)(Timing->AddressSetupTime                  |\
                    ((Timing->AddressHoldTime) << 4U)          |\
                    ((Timing->DataSetupTime) << 8U)            |\
                    ((Timing->BusTurnAroundDuration) << 16U)   |\
                    (((Timing->CLKDivision)-1U) << 20U)        |\
                    (((Timing->DataLatency)-2U) << 24U)        |\
                    (Timing->AccessMode));
  
  Device->BTCR[Bank + 1] = tmpr; 

#if defined(STM32F412Zx) || defined(STM32F412Vx) || defined(STM32F412Rx) || defined(STM32F413xx) || defined(STM32F423xx)
  /* Configure Clock division value (in NORSRAM bank 1) when continuous clock is enabled */
  if(HAL_IS_BIT_SET(Device->BTCR[FSMC_NORSRAM_BANK1], FSMC_BCR1_CCLKEN))
  {
    tmpr = (uint32_t)(Device->BTCR[FSMC_NORSRAM_BANK1 + 1U] & ~(0x0FU << 20U)); 
    tmpr |= (uint32_t)(((Timing->CLKDivision)-1U) << 20U);
    Device->BTCR[FSMC_NORSRAM_BANK1 + 1U] = tmpr;
  }
#endif /* STM32F412Zx || STM32F412Vx || STM32F412Rx || STM32F413xx || STM32F423xx */

}

/**
  * @brief  Initialize the FSMC_NORSRAM Extended mode Timing according to the specified
  *         parameters in the FSMC_NORSRAM_TimingTypeDef
  * @param  Device Pointer to NORSRAM device instance
  * @param  Timing Pointer to NORSRAM Timing structure
  * @param  Bank NORSRAM bank number  
  * @retval HAL status
  */
void  FSMC_NORSRAM_Extended_Timing_Init(FSMC_NORSRAM_EXTENDED_TypeDef *Device, FSMC_NORSRAM_TimingTypeDef *Timing, uint32_t Bank, uint32_t ExtendedMode)
{
  uint32_t tmpr = 0U;
  
  /* Check the parameters */
  assert_param(IS_FSMC_EXTENDED_MODE(ExtendedMode));

  /* Set NORSRAM device timing register for write configuration, if extended mode is used */
  if(ExtendedMode == FSMC_EXTENDED_MODE_ENABLE)
  {
    /* Check the parameters */
    assert_param(IS_FSMC_NORSRAM_EXTENDED_DEVICE(Device));
    assert_param(IS_FSMC_ADDRESS_SETUP_TIME(Timing->AddressSetupTime));
    assert_param(IS_FSMC_ADDRESS_HOLD_TIME(Timing->AddressHoldTime));
    assert_param(IS_FSMC_DATASETUP_TIME(Timing->DataSetupTime));
    assert_param(IS_FSMC_TURNAROUND_TIME(Timing->BusTurnAroundDuration));
    assert_param(IS_FSMC_ACCESS_MODE(Timing->AccessMode));
    assert_param(IS_FSMC_NORSRAM_BANK(Bank));
  
    /* Get the BWTR register value */
    tmpr = Device->BWTR[Bank];
    
    /* Clear ADDSET, ADDHLD, DATAST, BUSTURN and ACCMOD bits */
    tmpr &= ((uint32_t)~(FSMC_BWTR1_ADDSET  | FSMC_BWTR1_ADDHLD | FSMC_BWTR1_DATAST | \
                         FSMC_BWTR1_BUSTURN | FSMC_BWTR1_ACCMOD));

    tmpr |= (uint32_t)(Timing->AddressSetupTime                  |\
                      ((Timing->AddressHoldTime) << 4U)          |\
                      ((Timing->DataSetupTime) << 8U)            |\
                      ((Timing->BusTurnAroundDuration) << 16U)   |\
                      (Timing->AccessMode));
    
    Device->BWTR[Bank] = tmpr;
  }
  else                                        
  {
    Device->BWTR[Bank] = 0x0FFFFFFFU;
  }   
  
}

	
	
/* Private typedef -----------------------------------------------------------*/
/* Private define ------------------------------------------------------------*/

/* --------------------- FSMC registers bit mask ---------------------------- */
/* FSMC BCRx Mask */

/* FSMC PCRx Mask */
#define PCR_PBKEN_SET          ((uint32_t)0x00000004)
#define PCR_PBKEN_RESET        ((uint32_t)0x000FFFFB)
#define PCR_ECCEN_SET          ((uint32_t)0x00000040)
#define PCR_ECCEN_RESET        ((uint32_t)0x000FFFBF)
#define PCR_MEMORYTYPE_NAND    ((uint32_t)0x00000008)


/**
  * @}
  */

/** @defgroup FSMC_Group2 NAND Controller functions
 *  @brief   NAND Controller functions 
 *
@verbatim   
 ===============================================================================
                    NAND Controller functions
 ===============================================================================  

 The following sequence should be followed to configure the FSMC to interface with
 8-bit or 16-bit NAND memory connected to the NAND Bank:
 
   1. Enable the clock for the FSMC and associated GPIOs using the following functions:
          RCC_AHB3PeriphClockCmd(RCC_AHB3Periph_FSMC, ENABLE);
          RCC_AHB1PeriphClockCmd(RCC_AHB1Periph_GPIOx, ENABLE);

   2. FSMC pins configuration 
       - Connect the involved FSMC pins to AF12 using the following function 
          GPIO_PinAFConfig(GPIOx, GPIO_PinSourcex, GPIO_AF_FSMC); 
       - Configure these FSMC pins in alternate function mode by calling the function
          GPIO_Init();    
       
   3. Declare a FSMC_NANDInitTypeDef structure, for example:
          FSMC_NANDInitTypeDef  FSMC_NANDInitStructure;
      and fill the FSMC_NANDInitStructure variable with the allowed values of
      the structure member.
      
   4. Initialize the NAND Controller by calling the function
          FSMC_NANDInit(&FSMC_NANDInitStructure); 

   5. Then enable the NAND Bank, for example:
          FSMC_NANDCmd(FSMC_Bank3_NAND, ENABLE);  

   6. At this stage you can read/write from/to the memory connected to the NAND Bank. 
   
@note To enable the Error Correction Code (ECC), you have to use the function
          FSMC_NANDECCCmd(FSMC_Bank3_NAND, ENABLE);  
      and to get the current ECC value you have to use the function
          ECCval = FSMC_GetECC(FSMC_Bank3_NAND); 

@endverbatim
  * @{
  */
  
/**
  * @brief  Deinitializes the FSMC NAND Banks registers to their default reset values.
  * @param  FSMC_Bank: specifies the FSMC Bank to be used
  *          This parameter can be one of the following values:
  *            @arg FSMC_Bank2_NAND: FSMC Bank2 NAND 
  *            @arg FSMC_Bank3_NAND: FSMC Bank3 NAND 
  * @retval None
  */
void FSMC_NANDDeInit(uint32_t FSMC_Bank)
{
  /* Check the parameter */
  assert_param(IS_FSMC_NAND_BANK(FSMC_Bank));
  
  if(FSMC_Bank == FSMC_Bank2_NAND)
  {
    /* Set the FSMC_Bank2 registers to their reset values */
    FSMC_Bank2->PCR2 = 0x00000018;
    FSMC_Bank2->SR2 = 0x00000040;
    FSMC_Bank2->PMEM2 = 0xFCFCFCFC;
    FSMC_Bank2->PATT2 = 0xFCFCFCFC;  
  }
  /* FSMC_Bank3_NAND */  
  else
  {
    /* Set the FSMC_Bank3 registers to their reset values */
    FSMC_Bank3->PCR3 = 0x00000018;
    FSMC_Bank3->SR3 = 0x00000040;
    FSMC_Bank3->PMEM3 = 0xFCFCFCFC;
    FSMC_Bank3->PATT3 = 0xFCFCFCFC; 
  }  
}

/**
  * @brief  Initializes the FSMC NAND Banks according to the specified parameters
  *         in the FSMC_NANDInitStruct.
  * @param  FSMC_NANDInitStruct : pointer to a FSMC_NANDInitTypeDef structure that
  *         contains the configuration information for the FSMC NAND specified Banks.                       
  * @retval None
  */
void FSMC_NANDInit(FSMC_NANDInitTypeDef* FSMC_NANDInitStruct)
{
  uint32_t tmppcr = 0x00000000, tmppmem = 0x00000000, tmppatt = 0x00000000; 
    
  /* Check the parameters */
  assert_param( IS_FSMC_NAND_BANK(FSMC_NANDInitStruct->FSMC_Bank));
  assert_param( IS_FSMC_WAIT_FEATURE(FSMC_NANDInitStruct->FSMC_Waitfeature));
  assert_param( IS_FSMC_MEMORY_WIDTH(FSMC_NANDInitStruct->FSMC_MemoryDataWidth));
  assert_param( IS_FSMC_ECC_STATE(FSMC_NANDInitStruct->FSMC_ECC));
  assert_param( IS_FSMC_ECCPAGE_SIZE(FSMC_NANDInitStruct->FSMC_ECCPageSize));
  assert_param( IS_FSMC_TCLR_TIME(FSMC_NANDInitStruct->FSMC_TCLRSetupTime));
  assert_param( IS_FSMC_TAR_TIME(FSMC_NANDInitStruct->FSMC_TARSetupTime));
  assert_param(IS_FSMC_SETUP_TIME(FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_SetupTime));
  assert_param(IS_FSMC_WAIT_TIME(FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_WaitSetupTime));
  assert_param(IS_FSMC_HOLD_TIME(FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HoldSetupTime));
  assert_param(IS_FSMC_HIZ_TIME(FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HiZSetupTime));
  assert_param(IS_FSMC_SETUP_TIME(FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_SetupTime));
  assert_param(IS_FSMC_WAIT_TIME(FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_WaitSetupTime));
  assert_param(IS_FSMC_HOLD_TIME(FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HoldSetupTime));
  assert_param(IS_FSMC_HIZ_TIME(FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HiZSetupTime));
  
  /* Set the tmppcr value according to FSMC_NANDInitStruct parameters */
  tmppcr = (uint32_t)FSMC_NANDInitStruct->FSMC_Waitfeature |
            PCR_MEMORYTYPE_NAND |
            FSMC_NANDInitStruct->FSMC_MemoryDataWidth |
            FSMC_NANDInitStruct->FSMC_ECC |
            FSMC_NANDInitStruct->FSMC_ECCPageSize |
            (FSMC_NANDInitStruct->FSMC_TCLRSetupTime << 9 )|
            (FSMC_NANDInitStruct->FSMC_TARSetupTime << 13);
            
  /* Set tmppmem value according to FSMC_CommonSpaceTimingStructure parameters */
  tmppmem = (uint32_t)FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_SetupTime |
            (FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_WaitSetupTime << 8) |
            (FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HoldSetupTime << 16)|
            (FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HiZSetupTime << 24); 
            
  /* Set tmppatt value according to FSMC_AttributeSpaceTimingStructure parameters */
  tmppatt = (uint32_t)FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_SetupTime |
            (FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_WaitSetupTime << 8) |
            (FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HoldSetupTime << 16)|
            (FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HiZSetupTime << 24);
  
  if(FSMC_NANDInitStruct->FSMC_Bank == FSMC_Bank2_NAND)
  {
    /* FSMC_Bank2_NAND registers configuration */
    FSMC_Bank2->PCR2 = tmppcr;
    FSMC_Bank2->PMEM2 = tmppmem;
    FSMC_Bank2->PATT2 = tmppatt;
  }
  else
  {
    /* FSMC_Bank3_NAND registers configuration */
    FSMC_Bank3->PCR3 = tmppcr;
    FSMC_Bank3->PMEM3 = tmppmem;
    FSMC_Bank3->PATT3 = tmppatt;
  }
}


/**
  * @brief  Fills each FSMC_NANDInitStruct member with its default value.
  * @param  FSMC_NANDInitStruct: pointer to a FSMC_NANDInitTypeDef structure which
  *         will be initialized.
  * @retval None
  */
void FSMC_NANDStructInit(FSMC_NANDInitTypeDef* FSMC_NANDInitStruct)
{ 
  /* Reset NAND Init structure parameters values */
  FSMC_NANDInitStruct->FSMC_Bank = FSMC_Bank2_NAND;
  FSMC_NANDInitStruct->FSMC_Waitfeature = FSMC_Waitfeature_Disable;
  FSMC_NANDInitStruct->FSMC_MemoryDataWidth = FSMC_MemoryDataWidth_8b;
  FSMC_NANDInitStruct->FSMC_ECC = FSMC_ECC_Disable;
  FSMC_NANDInitStruct->FSMC_ECCPageSize = FSMC_ECCPageSize_256Bytes;
  FSMC_NANDInitStruct->FSMC_TCLRSetupTime = 0x0;
  FSMC_NANDInitStruct->FSMC_TARSetupTime = 0x0;
  FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_SetupTime = 0xFC;
  FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_WaitSetupTime = 0xFC;
  FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HoldSetupTime = 0xFC;
  FSMC_NANDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HiZSetupTime = 0xFC;
  FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_SetupTime = 0xFC;
  FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_WaitSetupTime = 0xFC;
  FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HoldSetupTime = 0xFC;
  FSMC_NANDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HiZSetupTime = 0xFC;	  
}

/**
  * @brief  Enables or disables the specified NAND Memory Bank.
  * @param  FSMC_Bank: specifies the FSMC Bank to be used
  *          This parameter can be one of the following values:
  *            @arg FSMC_Bank2_NAND: FSMC Bank2 NAND 
  *            @arg FSMC_Bank3_NAND: FSMC Bank3 NAND
  * @param  NewState: new state of the FSMC_Bank. This parameter can be: ENABLE or DISABLE.
  * @retval None
  */
void FSMC_NANDCmd(uint32_t FSMC_Bank, FunctionalState NewState)
{
  assert_param(IS_FSMC_NAND_BANK(FSMC_Bank));
  assert_param(IS_FUNCTIONAL_STATE(NewState));
  
  if (NewState != DISABLE)
  {
    /* Enable the selected NAND Bank by setting the PBKEN bit in the PCRx register */
    if(FSMC_Bank == FSMC_Bank2_NAND)
    {
      FSMC_Bank2->PCR2 |= PCR_PBKEN_SET;
    }
    else
    {
      FSMC_Bank3->PCR3 |= PCR_PBKEN_SET;
    }
  }
  else
  {
    /* Disable the selected NAND Bank by clearing the PBKEN bit in the PCRx register */
    if(FSMC_Bank == FSMC_Bank2_NAND)
    {
      FSMC_Bank2->PCR2 &= PCR_PBKEN_RESET;
    }
    else
    {
      FSMC_Bank3->PCR3 &= PCR_PBKEN_RESET;
    }
  }
}
/**
  * @brief  Enables or disables the FSMC NAND ECC feature.
  * @param  FSMC_Bank: specifies the FSMC Bank to be used
  *          This parameter can be one of the following values:
  *            @arg FSMC_Bank2_NAND: FSMC Bank2 NAND 
  *            @arg FSMC_Bank3_NAND: FSMC Bank3 NAND
  * @param  NewState: new state of the FSMC NAND ECC feature.  
  *          This parameter can be: ENABLE or DISABLE.
  * @retval None
  */
void FSMC_NANDECCCmd(uint32_t FSMC_Bank, FunctionalState NewState)
{
  assert_param(IS_FSMC_NAND_BANK(FSMC_Bank));
  assert_param(IS_FUNCTIONAL_STATE(NewState));
  
  if (NewState != DISABLE)
  {
    /* Enable the selected NAND Bank ECC function by setting the ECCEN bit in the PCRx register */
    if(FSMC_Bank == FSMC_Bank2_NAND)
    {
      FSMC_Bank2->PCR2 |= PCR_ECCEN_SET;
    }
    else
    {
      FSMC_Bank3->PCR3 |= PCR_ECCEN_SET;
    }
  }
  else
  {
    /* Disable the selected NAND Bank ECC function by clearing the ECCEN bit in the PCRx register */
    if(FSMC_Bank == FSMC_Bank2_NAND)
    {
      FSMC_Bank2->PCR2 &= PCR_ECCEN_RESET;
    }
    else
    {
      FSMC_Bank3->PCR3 &= PCR_ECCEN_RESET;
    }
  }
}

/**
  * @brief  Returns the error correction code register value.
  * @param  FSMC_Bank: specifies the FSMC Bank to be used
  *          This parameter can be one of the following values:
  *            @arg FSMC_Bank2_NAND: FSMC Bank2 NAND 
  *            @arg FSMC_Bank3_NAND: FSMC Bank3 NAND
  * @retval The Error Correction Code (ECC) value.
  */
uint32_t FSMC_GetECC(uint32_t FSMC_Bank)
{
  uint32_t eccval = 0x00000000;
  
  if(FSMC_Bank == FSMC_Bank2_NAND)
  {
    /* Get the ECCR2 register value */
    eccval = FSMC_Bank2->ECCR2;
  }
  else
  {
    /* Get the ECCR3 register value */
    eccval = FSMC_Bank3->ECCR3;
  }
  /* Return the error correction code value */
  return(eccval);
}
/**
  * @}
  */

/** @defgroup FSMC_Group3 PCCARD Controller functions
 *  @brief   PCCARD Controller functions 
 *
@verbatim   
 ===============================================================================
                    PCCARD Controller functions
 ===============================================================================  

 The following sequence should be followed to configure the FSMC to interface with
 16-bit PC Card compatible memory connected to the PCCARD Bank:
 
   1. Enable the clock for the FSMC and associated GPIOs using the following functions:
          RCC_AHB3PeriphClockCmd(RCC_AHB3Periph_FSMC, ENABLE);
          RCC_AHB1PeriphClockCmd(RCC_AHB1Periph_GPIOx, ENABLE);

   2. FSMC pins configuration 
       - Connect the involved FSMC pins to AF12 using the following function 
          GPIO_PinAFConfig(GPIOx, GPIO_PinSourcex, GPIO_AF_FSMC); 
       - Configure these FSMC pins in alternate function mode by calling the function
          GPIO_Init();    
       
   3. Declare a FSMC_PCCARDInitTypeDef structure, for example:
          FSMC_PCCARDInitTypeDef  FSMC_PCCARDInitStructure;
      and fill the FSMC_PCCARDInitStructure variable with the allowed values of
      the structure member.
      
   4. Initialize the PCCARD Controller by calling the function
          FSMC_PCCARDInit(&FSMC_PCCARDInitStructure); 

   5. Then enable the PCCARD Bank:
          FSMC_PCCARDCmd(ENABLE);  

   6. At this stage you can read/write from/to the memory connected to the PCCARD Bank. 
 
@endverbatim
  * @{
  */

/**
  * @brief  Deinitializes the FSMC PCCARD Bank registers to their default reset values.
  * @param  None                       
  * @retval None
  */
void FSMC_PCCARDDeInit(void)
{
  /* Set the FSMC_Bank4 registers to their reset values */
  FSMC_Bank4->PCR4 = 0x00000018; 
  FSMC_Bank4->SR4 = 0x00000000;	
  FSMC_Bank4->PMEM4 = 0xFCFCFCFC;
  FSMC_Bank4->PATT4 = 0xFCFCFCFC;
  FSMC_Bank4->PIO4 = 0xFCFCFCFC;
}

/**
  * @brief  Initializes the FSMC PCCARD Bank according to the specified parameters
  *         in the FSMC_PCCARDInitStruct.
  * @param  FSMC_PCCARDInitStruct : pointer to a FSMC_PCCARDInitTypeDef structure
  *         that contains the configuration information for the FSMC PCCARD Bank.                       
  * @retval None
  */
void FSMC_PCCARDInit(FSMC_PCCARDInitTypeDef* FSMC_PCCARDInitStruct)
{
  /* Check the parameters */
  assert_param(IS_FSMC_WAIT_FEATURE(FSMC_PCCARDInitStruct->FSMC_Waitfeature));
  assert_param(IS_FSMC_TCLR_TIME(FSMC_PCCARDInitStruct->FSMC_TCLRSetupTime));
  assert_param(IS_FSMC_TAR_TIME(FSMC_PCCARDInitStruct->FSMC_TARSetupTime));
 
  assert_param(IS_FSMC_SETUP_TIME(FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_SetupTime));
  assert_param(IS_FSMC_WAIT_TIME(FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_WaitSetupTime));
  assert_param(IS_FSMC_HOLD_TIME(FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HoldSetupTime));
  assert_param(IS_FSMC_HIZ_TIME(FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HiZSetupTime));
  
  assert_param(IS_FSMC_SETUP_TIME(FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_SetupTime));
  assert_param(IS_FSMC_WAIT_TIME(FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_WaitSetupTime));
  assert_param(IS_FSMC_HOLD_TIME(FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HoldSetupTime));
  assert_param(IS_FSMC_HIZ_TIME(FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HiZSetupTime));
  assert_param(IS_FSMC_SETUP_TIME(FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_SetupTime));
  assert_param(IS_FSMC_WAIT_TIME(FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_WaitSetupTime));
  assert_param(IS_FSMC_HOLD_TIME(FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_HoldSetupTime));
  assert_param(IS_FSMC_HIZ_TIME(FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_HiZSetupTime));
  
  /* Set the PCR4 register value according to FSMC_PCCARDInitStruct parameters */
  FSMC_Bank4->PCR4 = (uint32_t)FSMC_PCCARDInitStruct->FSMC_Waitfeature |
                     FSMC_MemoryDataWidth_16b |  
                     (FSMC_PCCARDInitStruct->FSMC_TCLRSetupTime << 9) |
                     (FSMC_PCCARDInitStruct->FSMC_TARSetupTime << 13);
            
  /* Set PMEM4 register value according to FSMC_CommonSpaceTimingStructure parameters */
  FSMC_Bank4->PMEM4 = (uint32_t)FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_SetupTime |
                      (FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_WaitSetupTime << 8) |
                      (FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HoldSetupTime << 16)|
                      (FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HiZSetupTime << 24); 
            
  /* Set PATT4 register value according to FSMC_AttributeSpaceTimingStructure parameters */
  FSMC_Bank4->PATT4 = (uint32_t)FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_SetupTime |
                      (FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_WaitSetupTime << 8) |
                      (FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HoldSetupTime << 16)|
                      (FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HiZSetupTime << 24);	
            
  /* Set PIO4 register value according to FSMC_IOSpaceTimingStructure parameters */
  FSMC_Bank4->PIO4 = (uint32_t)FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_SetupTime |
                     (FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_WaitSetupTime << 8) |
                     (FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_HoldSetupTime << 16)|
                     (FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_HiZSetupTime << 24);             
}

/**
  * @brief  Fills each FSMC_PCCARDInitStruct member with its default value.
  * @param  FSMC_PCCARDInitStruct: pointer to a FSMC_PCCARDInitTypeDef structure
  *         which will be initialized.
  * @retval None
  */
void FSMC_PCCARDStructInit(FSMC_PCCARDInitTypeDef* FSMC_PCCARDInitStruct)
{
  /* Reset PCCARD Init structure parameters values */
  FSMC_PCCARDInitStruct->FSMC_Waitfeature = FSMC_Waitfeature_Disable;
  FSMC_PCCARDInitStruct->FSMC_TCLRSetupTime = 0x0;
  FSMC_PCCARDInitStruct->FSMC_TARSetupTime = 0x0;
  FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_SetupTime = 0xFC;
  FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_WaitSetupTime = 0xFC;
  FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HoldSetupTime = 0xFC;
  FSMC_PCCARDInitStruct->FSMC_CommonSpaceTimingStruct->FSMC_HiZSetupTime = 0xFC;
  FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_SetupTime = 0xFC;
  FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_WaitSetupTime = 0xFC;
  FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HoldSetupTime = 0xFC;
  FSMC_PCCARDInitStruct->FSMC_AttributeSpaceTimingStruct->FSMC_HiZSetupTime = 0xFC;	
  FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_SetupTime = 0xFC;
  FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_WaitSetupTime = 0xFC;
  FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_HoldSetupTime = 0xFC;
  FSMC_PCCARDInitStruct->FSMC_IOSpaceTimingStruct->FSMC_HiZSetupTime = 0xFC;
}

/**
  * @brief  Enables or disables the PCCARD Memory Bank.
  * @param  NewState: new state of the PCCARD Memory Bank.  
  *          This parameter can be: ENABLE or DISABLE.
  * @retval None
  */
void FSMC_PCCARDCmd(FunctionalState NewState)
{
  assert_param(IS_FUNCTIONAL_STATE(NewState));
  
  if (NewState != DISABLE)
  {
    /* Enable the PCCARD Bank by setting the PBKEN bit in the PCR4 register */
    FSMC_Bank4->PCR4 |= PCR_PBKEN_SET;
  }
  else
  {
    /* Disable the PCCARD Bank by clearing the PBKEN bit in the PCR4 register */
    FSMC_Bank4->PCR4 &= PCR_PBKEN_RESET;
  }
}
/**
  * @}
  */

/** @defgroup FSMC_Group4  Interrupts and flags management functions
 *  @brief    Interrupts and flags management functions
 *
@verbatim   
 ===============================================================================
                     Interrupts and flags management functions
 ===============================================================================  

@endverbatim
  * @{
  */

/**
  * @brief  Enables or disables the specified FSMC interrupts.
  * @param  FSMC_Bank: specifies the FSMC Bank to be used
  *          This parameter can be one of the following values:
  *            @arg FSMC_Bank2_NAND: FSMC Bank2 NAND 
  *            @arg FSMC_Bank3_NAND: FSMC Bank3 NAND
  *            @arg FSMC_Bank4_PCCARD: FSMC Bank4 PCCARD
  * @param  FSMC_IT: specifies the FSMC interrupt sources to be enabled or disabled.
  *          This parameter can be any combination of the following values:
  *            @arg FSMC_IT_RisingEdge: Rising edge detection interrupt. 
  *            @arg FSMC_IT_Level: Level edge detection interrupt.
  *            @arg FSMC_IT_FallingEdge: Falling edge detection interrupt.
  * @param  NewState: new state of the specified FSMC interrupts.
  *          This parameter can be: ENABLE or DISABLE.
  * @retval None
  */
void FSMC_ITConfig(uint32_t FSMC_Bank, uint32_t FSMC_IT, FunctionalState NewState)
{
  assert_param(IS_FSMC_IT_BANK(FSMC_Bank));
  assert_param(IS_FSMC_IT(FSMC_IT));	
  assert_param(IS_FUNCTIONAL_STATE(NewState));
  
  if (NewState != DISABLE)
  {
    /* Enable the selected FSMC_Bank2 interrupts */
    if(FSMC_Bank == FSMC_Bank2_NAND)
    {
      FSMC_Bank2->SR2 |= FSMC_IT;
    }
    /* Enable the selected FSMC_Bank3 interrupts */
    else if (FSMC_Bank == FSMC_Bank3_NAND)
    {
      FSMC_Bank3->SR3 |= FSMC_IT;
    }
    /* Enable the selected FSMC_Bank4 interrupts */
    else
    {
      FSMC_Bank4->SR4 |= FSMC_IT;    
    }
  }
  else
  {
    /* Disable the selected FSMC_Bank2 interrupts */
    if(FSMC_Bank == FSMC_Bank2_NAND)
    {
      
      FSMC_Bank2->SR2 &= (uint32_t)~FSMC_IT;
    }
    /* Disable the selected FSMC_Bank3 interrupts */
    else if (FSMC_Bank == FSMC_Bank3_NAND)
    {
      FSMC_Bank3->SR3 &= (uint32_t)~FSMC_IT;
    }
    /* Disable the selected FSMC_Bank4 interrupts */
    else
    {
      FSMC_Bank4->SR4 &= (uint32_t)~FSMC_IT;    
    }
  }
}

/**
  * @brief  Checks whether the specified FSMC flag is set or not.
  * @param  FSMC_Bank: specifies the FSMC Bank to be used
  *          This parameter can be one of the following values:
  *            @arg FSMC_Bank2_NAND: FSMC Bank2 NAND 
  *            @arg FSMC_Bank3_NAND: FSMC Bank3 NAND
  *            @arg FSMC_Bank4_PCCARD: FSMC Bank4 PCCARD
  * @param  FSMC_FLAG: specifies the flag to check.
  *          This parameter can be one of the following values:
  *            @arg FSMC_FLAG_RisingEdge: Rising edge detection Flag.
  *            @arg FSMC_FLAG_Level: Level detection Flag.
  *            @arg FSMC_FLAG_FallingEdge: Falling edge detection Flag.
  *            @arg FSMC_FLAG_FEMPT: Fifo empty Flag. 
  * @retval The new state of FSMC_FLAG (SET or RESET).
  */
FlagStatus FSMC_GetFlagStatus(uint32_t FSMC_Bank, uint32_t FSMC_FLAG)
{
  FlagStatus bitstatus = RESET;
  uint32_t tmpsr = 0x00000000;
  
  /* Check the parameters */
  assert_param(IS_FSMC_GETFLAG_BANK(FSMC_Bank));
  assert_param(IS_FSMC_GET_FLAG(FSMC_FLAG));
  
  if(FSMC_Bank == FSMC_Bank2_NAND)
  {
    tmpsr = FSMC_Bank2->SR2;
  }  
  else if(FSMC_Bank == FSMC_Bank3_NAND)
  {
    tmpsr = FSMC_Bank3->SR3;
  }
  /* FSMC_Bank4_PCCARD*/
  else
  {
    tmpsr = FSMC_Bank4->SR4;
  } 
  
  /* Get the flag status */
  if ((tmpsr & FSMC_FLAG) != (uint16_t)RESET )
  {
    bitstatus = SET;
  }
  else
  {
    bitstatus = RESET;
  }
  /* Return the flag status */
  return bitstatus;
}

/**
  * @brief  Clears the FSMC's pending flags.
  * @param  FSMC_Bank: specifies the FSMC Bank to be used
  *          This parameter can be one of the following values:
  *            @arg FSMC_Bank2_NAND: FSMC Bank2 NAND 
  *            @arg FSMC_Bank3_NAND: FSMC Bank3 NAND
  *            @arg FSMC_Bank4_PCCARD: FSMC Bank4 PCCARD
  * @param  FSMC_FLAG: specifies the flag to clear.
  *          This parameter can be any combination of the following values:
  *            @arg FSMC_FLAG_RisingEdge: Rising edge detection Flag.
  *            @arg FSMC_FLAG_Level: Level detection Flag.
  *            @arg FSMC_FLAG_FallingEdge: Falling edge detection Flag.
  * @retval None
  */
void FSMC_ClearFlag(uint32_t FSMC_Bank, uint32_t FSMC_FLAG)
{
 /* Check the parameters */
  assert_param(IS_FSMC_GETFLAG_BANK(FSMC_Bank));
  assert_param(IS_FSMC_CLEAR_FLAG(FSMC_FLAG)) ;
    
  if(FSMC_Bank == FSMC_Bank2_NAND)
  {
    FSMC_Bank2->SR2 &= ~FSMC_FLAG; 
  }  
  else if(FSMC_Bank == FSMC_Bank3_NAND)
  {
    FSMC_Bank3->SR3 &= ~FSMC_FLAG;
  }
  /* FSMC_Bank4_PCCARD*/
  else
  {
    FSMC_Bank4->SR4 &= ~FSMC_FLAG;
  }
}

/**
  * @brief  Checks whether the specified FSMC interrupt has occurred or not.
  * @param  FSMC_Bank: specifies the FSMC Bank to be used
  *          This parameter can be one of the following values:
  *            @arg FSMC_Bank2_NAND: FSMC Bank2 NAND 
  *            @arg FSMC_Bank3_NAND: FSMC Bank3 NAND
  *            @arg FSMC_Bank4_PCCARD: FSMC Bank4 PCCARD
  * @param  FSMC_IT: specifies the FSMC interrupt source to check.
  *          This parameter can be one of the following values:
  *            @arg FSMC_IT_RisingEdge: Rising edge detection interrupt. 
  *            @arg FSMC_IT_Level: Level edge detection interrupt.
  *            @arg FSMC_IT_FallingEdge: Falling edge detection interrupt. 
  * @retval The new state of FSMC_IT (SET or RESET).
  */
ITStatus FSMC_GetITStatus(uint32_t FSMC_Bank, uint32_t FSMC_IT)
{
  ITStatus bitstatus = RESET;
  uint32_t tmpsr = 0x0, itstatus = 0x0, itenable = 0x0; 
  
  /* Check the parameters */
  assert_param(IS_FSMC_IT_BANK(FSMC_Bank));
  assert_param(IS_FSMC_GET_IT(FSMC_IT));
  
  if(FSMC_Bank == FSMC_Bank2_NAND)
  {
    tmpsr = FSMC_Bank2->SR2;
  }  
  else if(FSMC_Bank == FSMC_Bank3_NAND)
  {
    tmpsr = FSMC_Bank3->SR3;
  }
  /* FSMC_Bank4_PCCARD*/
  else
  {
    tmpsr = FSMC_Bank4->SR4;
  } 
  
  itstatus = tmpsr & FSMC_IT;
  
  itenable = tmpsr & (FSMC_IT >> 3);
  if ((itstatus != (uint32_t)RESET)  && (itenable != (uint32_t)RESET))
  {
    bitstatus = SET;
  }
  else
  {
    bitstatus = RESET;
  }
  return bitstatus; 
}

/**
  * @brief  Clears the FSMC's interrupt pending bits.
  * @param  FSMC_Bank: specifies the FSMC Bank to be used
  *          This parameter can be one of the following values:
  *            @arg FSMC_Bank2_NAND: FSMC Bank2 NAND 
  *            @arg FSMC_Bank3_NAND: FSMC Bank3 NAND
  *            @arg FSMC_Bank4_PCCARD: FSMC Bank4 PCCARD
  * @param  FSMC_IT: specifies the interrupt pending bit to clear.
  *          This parameter can be any combination of the following values:
  *            @arg FSMC_IT_RisingEdge: Rising edge detection interrupt. 
  *            @arg FSMC_IT_Level: Level edge detection interrupt.
  *            @arg FSMC_IT_FallingEdge: Falling edge detection interrupt.
  * @retval None
  */
void FSMC_ClearITPendingBit(uint32_t FSMC_Bank, uint32_t FSMC_IT)
{
  /* Check the parameters */
  assert_param(IS_FSMC_IT_BANK(FSMC_Bank));
  assert_param(IS_FSMC_IT(FSMC_IT));
    
  if(FSMC_Bank == FSMC_Bank2_NAND)
  {
    FSMC_Bank2->SR2 &= ~(FSMC_IT >> 3); 
  }  
  else if(FSMC_Bank == FSMC_Bank3_NAND)
  {
    FSMC_Bank3->SR3 &= ~(FSMC_IT >> 3);
  }
  /* FSMC_Bank4_PCCARD*/
  else
  {
    FSMC_Bank4->SR4 &= ~(FSMC_IT >> 3);
  }
}

/**
  * @}
  */ 

/**
  * @}
  */ 

/**
  * @}
  */

/**
  * @}
  */

/******************* (C) COPYRIGHT 2011 STMicroelectronics *****END OF FILE****/
