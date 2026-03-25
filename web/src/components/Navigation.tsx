import * as React from "react";
import { NavigationMenu } from "@base-ui/react";

const baseUrl = "/koruma";

export function Navigation() {
  return (
    <nav className="flex w-full justify-center px-8 py-4">
      <NavigationMenu.Root className="relative flex w-full justify-center">
        <NavigationMenu.List className="bg-bg-card border-accent/30 flex list-none gap-1 rounded-lg border p-1">
          <NavigationMenu.Item>
            <NavigationMenu.Link
              href={baseUrl}
              className="text-text hover:text-primary hover:bg-bg-hover block rounded-md px-4 py-2 no-underline transition-colors"
            >
              Home
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item>
            <NavigationMenu.Link
              href={`${baseUrl}/demos`}
              className="text-text hover:text-primary hover:bg-bg-hover block rounded-md px-4 py-2 no-underline transition-colors"
            >
              Demos
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item>
            <NavigationMenu.Link
              href={`${baseUrl}/docs`}
              className="text-text hover:text-primary hover:bg-bg-hover block rounded-md px-4 py-2 no-underline transition-colors"
            >
              Docs
            </NavigationMenu.Link>
          </NavigationMenu.Item>
        </NavigationMenu.List>
      </NavigationMenu.Root>
    </nav>
  );
}
