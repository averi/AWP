// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

import React from 'react';

// Main Table wrapper component
export const Table = ({ children, className = '' }) => (
  <div className={`w-full overflow-x-auto ${className}`}>
    <table className="w-full text-sm text-left text-gray-500">
      {children}
    </table>
  </div>
);

// Table Header (thead) component
export const TableHeader = ({ children, className = '' }) => (
  <thead className={`text-xs text-gray-700 uppercase bg-gray-50 ${className}`}>
    {children}
  </thead>
);

// Table Row (tr) component
export const TableRow = ({ children, className = '' }) => (
  <tr className={`bg-white border-b hover:bg-gray-50 ${className}`}>
    {children}
  </tr>
);

// Table Head Cell (th) component
export const TableHead = ({ children, className = '' }) => (
  <th scope="col" className={`px-4 py-3 ${className}`}>
    {children}
  </th>
);

// Table Body (tbody) component
export const TableBody = ({ children, className = '' }) => (
  <tbody className={className}>
    {children}
  </tbody>
);

// Table Data Cell (td) component
export const TableCell = ({ children, className = '' }) => (
  <td className={`px-4 py-3 ${className}`}>
    {children}
  </td>
);