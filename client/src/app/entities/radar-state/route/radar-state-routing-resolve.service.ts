import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, Router } from '@angular/router';

import { EMPTY, Observable, catchError, of } from 'rxjs';

import { IRadarState } from '../radar-state.model';
import { RadarStateService } from '../service/radar-state.service';

const radarStateResolve = (route: ActivatedRouteSnapshot): Observable<null | IRadarState> => {
  const { id } = route.params;
  if (id) {
    const router = inject(Router);
    const service = inject(RadarStateService);
    return service.find(id).pipe(
      catchError((error: HttpErrorResponse) => {
        if (error.status === 404) {
          router.navigate(['404']);
        } else {
          router.navigate(['error']);
        }
        return EMPTY;
      }),
    );
  }

  return of(null);
};

export default radarStateResolve;
