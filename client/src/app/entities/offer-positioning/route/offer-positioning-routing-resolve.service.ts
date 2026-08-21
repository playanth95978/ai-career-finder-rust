import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, Router } from '@angular/router';

import { EMPTY, Observable, catchError, of } from 'rxjs';

import { IOfferPositioning } from '../offer-positioning.model';
import { OfferPositioningService } from '../service/offer-positioning.service';

const offerPositioningResolve = (route: ActivatedRouteSnapshot): Observable<null | IOfferPositioning> => {
  const { id } = route.params;
  if (id) {
    const router = inject(Router);
    const service = inject(OfferPositioningService);
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

export default offerPositioningResolve;
