/*
 * Memory layout:
 *
 * FLASH is persistent and read-only (as far as our program is concerned). It
 * stores our code and initializers for static variables. Technically, it starts
 * at memory address 0, but the first 16K are reserved for the bootloader which
 * lets us flash new images using the Arduino software.
 *
 * While it's not possible to overwrite the bootloader (without using special
 * hardware to flash an image), we need to account for it to make sure that the
 * memory addresses in our program are correct. Therefore, we start the FLASH
 * area at address 0x4000.
 *
 * RAM should be self-explanatory, we have 32K of it starting at 0x20000000.
 */
MEMORY
{
	FLASH : ORIGIN = 0x4000, LENGTH = 256K - 16K
	RAM : ORIGIN = 0x20000000, LENGTH = 32K
}

/*
 * Mark the Reset function as entry-point. This doesn't really do anything
 * because the function that the CPU starts executing is given in the vector
 * table below. However, the linker can discard code if it isn't reachable from
 * the entry point, so we specify Reset as the entry point.
 */
ENTRY(Reset);

SECTIONS
{
	/*
	 * ARMv7M CPUs expect a "vector table" at the start of FLASH.
	 *
	 * The first entry is the initial stack pointer value. The rest are function
	 * pointers for various exception and interrupt handlers.
	 *
	 * The second entry is especially important: It is the reset handler which is
	 * executed when powering on and when pressing the reset button.
	 */
	.vector_table ORIGIN(FLASH) :
	{
		/* Initial stack pointer value. */
		LONG(ORIGIN(RAM) + LENGTH(RAM));

		/* KEEP ensures that they can't be optimized away. */
		KEEP(*(.vector_table.reset_vector));
		KEEP(*(.vector_table.exceptions));
		KEEP(*(.vector_table.external_interrupts));
	} > FLASH

	/* Program code. */
	.text :
	{
		*(.text .text.*);
	} > FLASH

	/* Read-only data, immutable static variables and string literals. */
	.rodata :
	{
		*(.rodata .rodata.*);
	} > FLASH

	/*
	 * Section for static variables initialized to 0. This section must be zeroized
	 * before calling main. We do this in initialize_ram().
	 *
	 * We define symbols _sbss and _ebss for the start and end of this section
	 * so that we know what part to zeroize.
	 */
	.bss :
	{
		_sbss = .;
		*(.bss .bss.*);
		_ebss = .;
	} > RAM

	/*
	 * The data section is for static variables with non-zero initializers.
	 * During execution, these values are in RAM, but since RAM is volatile,
	 * the initializers have to be stored in FLASH. The "AT > FLASH" at the
	 * end makes sure that this happens: The section has a load address in FLASH
	 * and a virtual memory address in RAM. It is the responsibility of the runtime
	 * to copy these initial values to the proper part of the RAM before calling
	 * the main function. We do this in the function initialize_ram().
	 *
	 * We define symbols _sdata and _edata to have the (virtual) addresses of the
	 * start and end of this section, so that we know where we need to copy the
	 * initial values.
	 */
	.data :
	{
		_sdata = .;
		*(.data .data.*);
		_edata = .;
	} > RAM AT > FLASH

	/*
	 * Define a symbol for the load address of the data section: The section in
	 * FLASH where the initial values for the data section are stored. That way,
	 * we know where to copy the initial data from in initialize_ram().
	 */
	_sidata = LOADADDR(.data);

	/* Throw away code for stack unwinding, we don't do that for now. */
	/DISCARD/ :
	{
		*(.ARM.exidx .ARM.exidx.*);
	}
}
