export const metadata = {
  title: 'Next.js API Benchmark Server',
  description: 'API benchmark server for framework comparison',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

