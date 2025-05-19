// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

import React from 'react';
import { ChevronDown } from 'lucide-react';

export const Select = ({ value, onChange, options = [], placeholder, disabled = false, className = '' }) => (
  <div className={`relative ${className}`}>
    <select
      value={value}
      onChange={onChange}
      disabled={disabled}
      className={`block w-full px-4 py-2 pr-8 text-sm text-gray-700 bg-white border border-gray-300 rounded-md shadow-sm appearance-none focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 ${disabled ? 'bg-gray-100 cursor-not-allowed' : ''}`}
    >
      {placeholder && <option value="" disabled>{placeholder}</option>}
      {options.map(option => (
        <option key={option.value} value={option.value}>{option.label}</option>
      ))}
    </select>
    <div className="absolute inset-y-0 right-0 flex items-center px-2 pointer-events-none">
      <ChevronDown className="w-4 h-4 text-gray-400" />
    </div>
  </div>
);