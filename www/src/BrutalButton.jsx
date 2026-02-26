import React from 'react';

const BrutalButton = ({ 
  children, 
  onClick, 
  variant = 'primary', 
  size = 'md', 
  className = '',
  type = 'button'
}) => {
  const baseStyles = 'inline-flex items-center justify-center font-bold transition-all duration-100 border-2 border-bio-black focus:outline-none active:translate-x-[2px] active:translate-y-[2px] active:shadow-none';
  
  const variants = {
    primary: 'bg-bio-green text-white shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] hover:shadow-[6px_6px_0px_0px_rgba(0,0,0,1)]',
    outline: 'bg-white text-bio-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] hover:shadow-[6px_6px_0px_0px_rgba(0,0,0,1)]',
    ghost: 'bg-transparent text-bio-black border-transparent hover:bg-bio-green/10',
    danger: 'bg-red-500 text-white shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] hover:shadow-[6px_6px_0px_0px_rgba(0,0,0,1)]'
  };

  const sizes = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-6 py-3 text-base',
    lg: 'px-8 py-4 text-lg'
  };

  return (
    <button
      type={type}
      onClick={onClick}
      className={`${baseStyles} ${variants[variant]} ${sizes[size]} ${className}`}
    >
      {children}
    </button>
  );
};

export default BrutalButton;
